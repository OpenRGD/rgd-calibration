#!/usr/bin/env python3
"""D.1: independently rerun owner candidates with both CAL binaries.

The A/B/C workflows and their historical pins remain unchanged. All observations
are synthetic. No installed registry, live API, device, or actuation.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from unittest import mock

from test_contract_exchange import (
    ASSOCIATION, CONSOLE_SCHEMA, IDENTITY, PASSIVE, POSITION, PROFILE, RI_SCHEMA,
    ROOT, rejected, run_json, sha256, validate_cal_exchange, validators, zip_bytes,
)

PINS = {
    "core": "afe1a831b950a4e6560f0285a0103fc4eaa343b7",
    "console": "168e99c2ba057ae78bfe4cbde7465c222eaa84de",
    "registry": "0268b3748042d04fd3561553616388368881e04a",
    "ri": "563d923cd65805a2fc40b431635241871720932f",
    "legacy_cal": "81700401e2fa6d370a86b6c1f5a48802bc5f1fc7",
}
CAL_C_HEAD = "646dbc05bcefd15942b25bb55b3c99096f0cccc9"
CORE_ROOT = "sha256:c049dc3fb91c19e668ef0d35b9b40d9052caf1a458b1a108d58b1896189dd5f9"
DESCRIPTION = "spec/01_foundation/description.jsonc"


def check_checkout(root, expected):
    head = subprocess.check_output(
        ["git", "-C", str(root), "rev-parse", "HEAD"], text=True
    ).strip()
    assert head == expected, (root, head, expected)
    assert not subprocess.check_output(
        ["git", "-C", str(root), "status", "--porcelain", "--untracked-files=no"],
        text=True,
    ).strip(), f"modified tracked checkout: {root}"


def blob_id(root, path, expected):
    actual = subprocess.check_output(
        ["git", "-C", str(root), "hash-object", path], text=True
    ).strip()
    assert actual == expected, (path, actual, expected)


def run_owner_tests(paths, legacy, binary, probe):
    # Execute the unmodified upstream owner test code, not a local rewrite.
    subprocess.run([
        sys.executable, "-B", str(paths["console"] / "scripts/test_calibration_consumer_compat.py"),
        "--cal", str(ROOT), "--registry", str(paths["registry"]),
        "--legacy-binary", str(legacy), "--candidate-binary", str(binary),
    ], check=True, timeout=180)
    env = dict(os.environ, PYTHONPATH=str(paths["ri"] / "src"),
               CAL_CHECKOUT=str(ROOT), CAL_PROBE=str(probe))
    subprocess.run([
        sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests_cal_external", "-v",
    ], cwd=paths["ri"], env=env, check=True, timeout=180)
    print("PASS owner suites: Console 22 syntax cases + 19 route cases per binary; RI 27 variants / 44 sidecars", flush=True)


def expect_agent_rejection(agent, callback, label):
    try:
        callback()
    except agent.InspectionError:
        return
    raise AssertionError(f"candidate Agent accepted {label}")


def inspect_binary(agent, binary, blob, folder):
    digest = sha256(binary.read_bytes())
    # The descriptor and safety policy come from the actual pinned Registry.
    # Only installed status/resolution are injected. No installer is executed.
    with mock.patch.object(agent.registry, "status_of", return_value={"status": "ready"}), \
         mock.patch.object(agent.registry, "resolve_binary", return_value={
             "binary": str(binary), "binary_sha256": digest,
         }):
        return agent.CalibrationInspector(folder).inspect(blob)


def check_console_core(paths, legacy, binary, v):
    sys.path.insert(0, str(paths["registry"]))
    sys.path.insert(0, str(paths["console"] / "services/module-agent"))
    import calibration_passive as agent
    import calibration_contract as guard
    from io import BytesIO
    import zipfile

    assert agent.MODULE_VERSION == "0.2.0"
    descriptor = agent.registry.entry(agent.MODULE_ID, agent.MODULE_VERSION)
    assert descriptor["source"]["commit_sha"] == PINS["legacy_cal"]
    assert descriptor["permissions_required"] == []
    assert descriptor["safety"] == {
        "non_actuating": True, "automatic_motion_on_load": False,
        "active_calibration_supported": False,
    }
    for schema in (PASSIVE, PROFILE):
        local = ROOT / "contracts" / schema
        owner_copy = paths["console"] / "services/module-agent/calibration-contracts" / schema
        assert local.read_bytes() == owner_copy.read_bytes(), schema

    # The same full pinned Core spec, once through the Core verifier and once
    # through Console's existing five-file extraction. Root and ZIP differ.
    identity = run_json([
        sys.executable, "-I", "-B", str(ROOT / "tools/canonical_identity/verify.py"),
        "--bundle", str(paths["core"]), "--core-checkout", str(paths["core"]),
        "--cal-binary", str(binary),
    ])
    v[IDENTITY].validate(identity)
    assert identity["integrity_status"] == "source_tree_hash_match"
    assert identity["computed_source_tree_sha256"] == CORE_ROOT
    assert identity["declared_source_tree_sha256"] == CORE_ROOT
    assert identity["indexed_file_count"] == 76
    assert identity["report_binding_status"] == "source_index_match"
    assert identity["bundle_completeness"] == "not_assessed"
    for key in ("canonical_schema_validated", "robot_binding_verified", "profile_model_hash_bound",
                "physical_calibration_certified", "actuation_enabled"):
        assert identity[key] is False, key
    v[PASSIVE].validate(identity["calibration_report"])
    blob = zip_bytes(paths["core"])
    original = sha256(blob)
    assert original != CORE_ROOT
    with tempfile.TemporaryDirectory(prefix="cal-d1-") as tmp:
        folder = Path(tmp)
        inspections = []
        for executable in (legacy, binary):
            inspected = inspect_binary(agent, executable, blob, folder)
            report = inspected["report"]
            v[PASSIVE].validate(report)
            assert inspected["binary_sha256"] == sha256(executable.read_bytes())
            assert inspected["module_version"] == "0.2.0"
            assert inspected["inspection_contract"] == "console.passive-calibration.v0.4b"
            assert inspected["actuation_enabled"] is False
            assert inspected["physical_calibration_certified"] is False
            assert report["source_digests"] == identity["calibration_report"]["source_digests"]
            assert report["coverage"] == identity["calibration_report"]["coverage"]
            assert len(report["source_digests"]) == 5
            assert report["profile_binding"] == "no_profile"
            assert report["profile_id"] is None and report["profile_state"] is None
            assert report["export_preview"] is None
            wrapper = {
                "schema_version": "console.calibration-inspection.v0.4b",
                "robot_id": "synthetic_d1_robot",
                "source_model_id": "synthetic_d1_model",
                "source_profile_sha256": original,
                "source_integrity": "canonical_not_verified",
                "generated_at": inspected["generated_at"],
                "evidence_class": "declared_profile_only_not_physical_observation",
                "physical_calibration_certified": False, "actuation_enabled": False,
                "inspection": inspected,
            }
            v[CONSOLE_SCHEMA].validate(wrapper)
            rejected(v[CONSOLE_SCHEMA], dict(wrapper, source_integrity="canonical_verified"),
                     "legacy wrapper canonical promotion")
            inspections.append(report)
        assert sha256(blob) == original
        assert inspections[0]["coverage"] == inspections[1]["coverage"]
        print("PASS actual Core reference through both CLI binaries and candidate Agent: 76-file identity, five exact raw source hashes, unchanged passive coverage")

        # Directly exercise the consumer validation boundary on the real
        # report and extraction. Mutated variants are not actual CLI output.
        source = agent.extract_foundation_zip(blob, folder / "private")
        report = inspections[1]
        agent.validate_cal02_report(report, source)
        bad = {}
        bad["missing_export_preview"] = {k: x for k, x in report.items() if k != "export_preview"}
        bad["fabricated_profile_binding"] = dict(report, profile_binding="claimed_match", profile_id="fake")
        bad["bad_source_digest"] = copy.deepcopy(report)
        bad["bad_source_digest"]["source_digests"][0]["raw_file_sha256"] = "bad"
        bad["wrong_valid_digest"] = copy.deepcopy(report)
        bad["wrong_valid_digest"]["source_digests"][0]["raw_file_sha256"] = "sha256:" + "0" * 64
        bad["missing_source"] = copy.deepcopy(report)
        bad["missing_source"]["source_digests"].pop()
        bad["duplicate_source_role"] = copy.deepcopy(report)
        bad["duplicate_source_role"]["source_digests"].append(
            copy.deepcopy(report["source_digests"][0]))
        bad["foreign_source_path"] = copy.deepcopy(report)
        bad["foreign_source_path"]["source_digests"][0]["path"] = "../never_read.jsonc"
        bad["bad_coverage"] = copy.deepcopy(report)
        bad["bad_coverage"]["coverage"]["candidate_fields"] += 1
        bad["physical_authority"] = dict(report, claims_physical_calibration_certified=True)
        bad["historical_measurements"] = copy.deepcopy(report)
        bad["historical_measurements"]["parameters"][0]["stored_measurements"].append({
            "parameter_ref": "synthetic_claim", "kind": "observed", "value": 0.0,
            "unit": "rad", "standard_uncertainty": 0.1, "evidence_ids": ["synthetic"],
        })
        for name, value in bad.items():
            expect_agent_rejection(agent, lambda: agent.validate_cal02_report(value, source), name)
        # A syntactically valid report with original hash is stale if the
        # private extraction no longer contains the exact original bytes.
        target = source.root / DESCRIPTION
        target.write_bytes(target.read_bytes() + b"\n// synthetic modification\n")
        expect_agent_rejection(agent, lambda: agent.validate_cal02_report(report, source),
                               "mutated private snapshot")
        print(f"PASS {len(bad) + 1} candidate consumer negative checks, including all three C-era gaps and private snapshot tamper")

        # The old route accepts only .jsonc. Do not hide any same-root JSON
        # sibling. The strict syntax guard works on scratch bytes only.
        raw = b'{"meta_group":{"model_id_str":"synthetic"},"joint_topology_map":{},"sensors_mounting_map":{}}'
        for name in ("description.json", "calibration.json"):
            stream = BytesIO()
            with zipfile.ZipFile(stream, "w") as archive:
                archive.writestr(DESCRIPTION, raw)
                archive.writestr("spec/01_foundation/" + name, b"{}")
            expect_agent_rejection(agent, lambda: agent.extract_foundation_zip(
                stream.getvalue(), folder / ("layout-" + name)), name)
        corpus = json.loads((ROOT / "tests/fixtures/cal-021-a/bundle-syntax-corpus.v1.json").read_text())
        cr = next(x for x in corpus["cases"] if x["id"] == "cr-comment")["raw"].encode("utf-8")
        assert b"\r" in cr
        guard.validate_cal_source(cr, legacy=False)
        try:
            guard.validate_cal_source(cr, legacy=True)
        except ValueError:
            pass
        else:
            raise AssertionError("legacy-compatible guard accepted CR-only comment")
        direct = folder / "direct-cr/spec/01_foundation"
        direct.mkdir(parents=True)
        (direct / "description.jsonc").write_bytes(cr)
        modern = run_json([str(binary), "inspect", "--bundle", str(direct.parents[1])])
        v[PASSIVE].validate(modern)
        old = subprocess.run([str(legacy), "inspect", "--bundle", str(direct.parents[1])],
                             capture_output=True, timeout=35, check=False)
        assert old.returncode != 0, "legacy parser behavior unexpectedly changed"
        assert (direct / "description.jsonc").read_bytes() == cr
        print("PASS two unsupported Foundation layout cases; CR-only comment is accepted by new CAL CLI and refused by the old binary/legacy route. D migration gate remains.")
    print("Core/Console full-source snapshot remains two independent local checks; no canonical promotion of Console wrapper or approved installed version.")


def main():
    parser = argparse.ArgumentParser()
    for key in PINS:
        parser.add_argument("--" + key.replace("_", "-"), dest=key, type=Path, required=True)
    for key in ("legacy_binary", "binary", "probe"):
        parser.add_argument("--" + key.replace("_", "-"), dest=key, type=Path, required=True)
    args = parser.parse_args()
    paths = {k: getattr(args, k).resolve(strict=True) for k in PINS}
    for name, path in paths.items():
        check_checkout(path, PINS[name])
    # Preserve the exact previously reviewed CAL code and RI producer shape.
    for path, expected in (
        ("contracts/" + RI_SCHEMA, "3d92e074e71593bf6db0a735394c628ea131668d"),
        ("src/reality_ingress/consumer_ref.py", "a34f4cbd924b58c849af215d175500f5a5ca2ece"),
    ):
        blob_id(paths["ri"], path, expected)
    for path, expected in (
        ("contracts/" + ASSOCIATION, "6c26a4d3618721326326a0cb2966a5137f6d87fe"),
        ("contracts/" + PASSIVE, "8c5080a7e36b728581caaa9332d7dea5e8659f41"),
        ("contracts/" + PROFILE, "30ba581d528d66011dca69d6ecd29da7ba1f22b5"),
        ("src/ri_association.rs", "815c20efc4acf60d943d7214455b85f871209bb3"),
    ):
        blob_id(ROOT, path, expected)
    assert (ROOT / "Cargo.toml").read_text().split('version = "0.2.0"', 1)[0].startswith("[package]")
    binary, legacy, probe = (getattr(args, k).resolve(strict=True) for k in ("binary", "legacy_binary", "probe"))
    schemas = {name: ROOT / "contracts" / name for name in (PROFILE, POSITION, PASSIVE, ASSOCIATION, IDENTITY)}
    schemas[RI_SCHEMA] = paths["ri"] / "contracts" / RI_SCHEMA
    schemas[CONSOLE_SCHEMA] = paths["console"] / "contracts/console/v1" / CONSOLE_SCHEMA
    v = validators(schemas)

    run_owner_tests(paths, legacy, binary, probe)
    # Repeat C's actual producer exchange against the newer RI stack. Keep
    # its historical gap assertions in the original C workflow unchanged.
    with tempfile.TemporaryDirectory(prefix="cal-d1-ri-") as tmp:
        validate_cal_exchange(probe, paths["ri"], v, Path(tmp))
    check_console_core(paths, legacy, binary, v)
    for name, path in paths.items():
        check_checkout(path, PINS[name])
    print("PASS D.1 candidate reconciliation: no Registry install/repin, live API/UI, A02 resolver activation, wire-contract change or hardware commissioning.")


if __name__ == "__main__":
    main()
