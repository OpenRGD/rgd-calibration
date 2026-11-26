"""Offline tests of CAL-021-B against the actual pinned upstream Core.

Set CAL_TEST_CORE and CAL_TEST_BINARY to explicitly trusted local paths.
All mutations are on disposable test copies, never the Core checkout.
"""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "tools/canonical_identity/verify.py"
CORE = Path(os.environ["CAL_TEST_CORE"]).resolve()
BINARY = Path(os.environ["CAL_TEST_BINARY"]).resolve()
EXPECTED_ROOT = "sha256:c049dc3fb91c19e668ef0d35b9b40d9052caf1a458b1a108d58b1896189dd5f9"
SCHEMA = json.loads((ROOT / "contracts/canonical-identity.v0.1-candidate.schema.json").read_text())
VALIDATOR = Draft202012Validator(SCHEMA)
SPEC = importlib.util.spec_from_file_location("cal_identity_test_module", SCRIPT)
ADAPTER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ADAPTER)


class CanonicalIdentityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bundle = self.root / "robot"
        self.bundle.mkdir()
        shutil.copytree(CORE / "spec", self.bundle / "spec")

    def run_assessment(self, expected_exit=0, bundle=None, checkout=None):
        run = subprocess.run(
            [sys.executable, "-I", "-B", str(SCRIPT),
             "--bundle", str(bundle or self.bundle),
             "--core-checkout", str(checkout or CORE),
             "--cal-binary", str(BINARY)],
            capture_output=True, timeout=90, check=False,
        )
        self.assertEqual(run.returncode, expected_exit, run.stderr.decode())
        result = json.loads(run.stdout)
        VALIDATOR.validate(result)
        self.assertFalse(result["canonical_schema_validated"])
        self.assertEqual(result["bundle_completeness"], "not_assessed")
        self.assertIsNone(result["physical_instance_id"])
        for flag in (
            "robot_binding_verified", "profile_model_hash_bound",
            "physical_calibration_certified", "actuation_enabled",
        ):
            self.assertIs(result[flag], False)
        return result

    def test_real_core_reference_matches_without_mutating_inputs(self):
        before = {
            p.relative_to(self.bundle): p.read_bytes()
            for p in self.bundle.rglob("*") if p.is_file()
        }
        result = self.run_assessment()
        self.assertEqual(result["integrity_status"], "source_tree_hash_match")
        self.assertEqual(result["declared_source_tree_sha256"], EXPECTED_ROOT)
        self.assertEqual(result["computed_source_tree_sha256"], EXPECTED_ROOT)
        self.assertEqual(result["indexed_file_count"], 76)
        self.assertTrue(result["core_code_pin_checked"])
        self.assertEqual(result["report_binding_status"], "source_index_match")
        self.assertEqual(result["calibration_report"]["model_id"], "bhl_humanoid_lite_v2")
        self.assertEqual(result["calibration_report"]["coverage"]["saved_evidence_fields"], 0)
        after = {
            p.relative_to(self.bundle): p.read_bytes()
            for p in self.bundle.rglob("*") if p.is_file()
        }
        self.assertEqual(before, after)

    def test_change_outside_foundation_invalidates_root(self):
        path = self.bundle / "spec/00_core/kernel.jsonc"
        path.write_bytes(path.read_bytes() + b"\n// synthetic tamper\n")
        result = self.run_assessment(2)
        self.assertEqual(result["integrity_status"], "source_tree_hash_mismatch")
        self.assertNotEqual(result["computed_source_tree_sha256"], EXPECTED_ROOT)
        self.assertEqual(result["declared_source_tree_sha256"], EXPECTED_ROOT)
        self.assertEqual(result["report_binding_status"], "source_index_match")

    def test_comment_change_in_inspected_document_changes_raw_digest_and_root(self):
        path = self.bundle / "spec/01_foundation/description.jsonc"
        path.write_bytes(path.read_bytes() + b"\n// comment changes exact bytes\n")
        result = self.run_assessment(2)
        self.assertEqual(result["integrity_status"], "source_tree_hash_mismatch")
        self.assertEqual(result["report_binding_status"], "source_index_match")
        digest = next(x for x in result["calibration_report"]["source_digests"] if x["role"] == "description")
        self.assertEqual(digest["raw_file_sha256"], ADAPTER.sha256(path.read_bytes()))

    def test_core_generated_top_level_file_is_excluded_by_core(self):
        path = self.bundle / "spec/openrgd_unified_spec.jsonc"
        path.write_text('{"generated":true}', encoding="utf-8")
        result = self.run_assessment()
        self.assertEqual(result["computed_source_tree_sha256"], EXPECTED_ROOT)
        self.assertEqual(result["indexed_file_count"], 76)

    def test_foundation_without_manifest_is_inspectable_not_verified(self):
        (self.bundle / "spec/manifest.jsonc").unlink()
        result = self.run_assessment(2)
        self.assertEqual(result["integrity_status"], "manifest_missing")
        self.assertEqual(result["report_binding_status"], "source_index_not_bound")
        self.assertIsNone(result["computed_source_tree_sha256"])
        self.assertIsNotNone(result["calibration_report"])

    def test_standard_only_is_not_a_canonical_source_tree(self):
        shutil.rmtree(self.bundle / "spec")
        shutil.copytree(CORE / "standard", self.bundle / "standard")
        result = self.run_assessment(2)
        self.assertEqual(result["integrity_status"], "source_tree_unavailable")
        self.assertIsNone(result["calibration_report"])

    def test_direct_spec_directory_supported(self):
        result = self.run_assessment(bundle=self.bundle / "spec")
        self.assertEqual(result["computed_source_tree_sha256"], EXPECTED_ROOT)

    def test_unknown_profile_does_not_compute_a_replacement_digest(self):
        path = self.bundle / "spec/manifest.jsonc"
        raw = path.read_text(encoding="utf-8")
        path.write_text(raw.replace(ADAPTER.PROFILE, "UNSUPPORTED_TEST_PROFILE"), encoding="utf-8")
        result = self.run_assessment(2)
        self.assertEqual(result["integrity_status"], "unsupported_profile")
        self.assertIsNone(result["computed_source_tree_sha256"])

    def test_ambiguous_manifest_self_reference_rejected(self):
        path = self.bundle / "spec/manifest.jsonc"
        raw = path.read_bytes()
        path.write_bytes(b'// "integrity_hash_str": "sha256:OTHER"\n' + raw)
        result = self.run_assessment(2)
        self.assertEqual(result["integrity_status"], "verification_failed")
        self.assertIn("manifest_self_hash_ambiguous", result["reasons"])

    def test_duplicate_manifest_member_rejected(self):
        path = self.bundle / "spec/manifest.jsonc"
        raw = path.read_text(encoding="utf-8")
        path.write_text(raw.replace('"meta_group": {', '"meta_group": {}, "meta_group": {', 1), encoding="utf-8")
        result = self.run_assessment(2)
        self.assertEqual(result["integrity_status"], "verification_failed")
        self.assertIn("manifest_duplicate_key", result["reasons"])

    def test_unindexed_json_document_cannot_be_bound_to_core(self):
        # Only a synthetic self-consistent small tree. Hash matching is NOT
        # completeness or schema validity. Official Core writes the test hash.
        spec = self.bundle / "spec"
        shutil.rmtree(spec)
        foundation = spec / "01_foundation"
        foundation.mkdir(parents=True)
        (foundation / "description.jsonc").write_text(
            '{"meta_group":{"model_id_str":"synthetic_partial"},"joint_topology_map":{},"sensors_mounting_map":{}}',
            encoding="utf-8",
        )
        (spec / "manifest.jsonc").write_text(json.dumps({
            "meta_group": {
                "integrity_profile_str": ADAPTER.PROFILE,
                "integrity_hash_str": "sha256:" + "0" * 64,
            }
        }), encoding="utf-8")
        # Import the exact Core implementation only for creating test data.
        # The assessment under test still runs in a separate fresh process.
        core = ADAPTER.load_pinned_core(CORE, self.root)
        sealed = core.update_manifest_integrity(spec)
        self.assertTrue(sealed.matches)
        matched = self.run_assessment()
        self.assertEqual(matched["integrity_status"], "source_tree_hash_match")
        self.assertEqual(matched["bundle_completeness"], "not_assessed")
        (foundation / "calibration.json").write_text("{}", encoding="utf-8")
        result = self.run_assessment(2)
        self.assertEqual(result["integrity_status"], "source_tree_hash_match")
        self.assertEqual(result["report_binding_status"], "source_index_not_bound")
        self.assertEqual(result["computed_source_tree_sha256"], matched["computed_source_tree_sha256"])

    def test_missing_or_altered_core_is_not_a_verified_authority(self):
        altered = self.root / "wrong-core"
        dst = altered / "src/openrgd/core"
        dst.mkdir(parents=True)
        for name in ADAPTER.CORE_BLOBS:
            shutil.copy2(CORE / "src/openrgd/core" / name, dst / name)
        path = dst / "canonical.py"
        path.write_bytes(path.read_bytes() + b"\n# test modification\n")
        result = self.run_assessment(2, checkout=altered)
        self.assertEqual(result["integrity_status"], "verifier_unavailable")
        self.assertFalse(result["core_code_pin_checked"])
        self.assertIsNone(result["computed_source_tree_sha256"])

    def test_symlink_is_rejected_before_core_or_cal_invocation(self):
        (self.bundle / "spec/link.jsonc").symlink_to(self.bundle / "spec/manifest.jsonc")
        result = self.run_assessment(2)
        self.assertEqual(result["integrity_status"], "verification_failed")
        self.assertIn("unsafe_source_entry", result["reasons"])
        self.assertFalse(result["core_code_pin_checked"])
        self.assertIsNone(result["calibration_report"])

    def test_cal_parser_failure_cannot_yield_bound_report(self):
        path = self.bundle / "spec/01_foundation/description.jsonc"
        path.write_text('{"meta_group":{},"meta_group":{}}', encoding="utf-8")
        result = self.run_assessment(2)
        self.assertEqual(result["report_binding_status"], "inspection_failed")
        self.assertIsNone(result["calibration_report"])

    def test_report_digest_mismatch_is_rejected(self):
        result = self.run_assessment()
        report = result["calibration_report"]
        report["source_digests"][0]["raw_file_sha256"] = "sha256:" + "a" * 64
        with self.assertRaisesRegex(ADAPTER.BoundaryError, "cal_report_source_digest_mismatch"):
            ADAPTER.match_report_to_index(report, self.bundle / "spec", None)


if __name__ == "__main__":
    unittest.main(verbosity=2)
