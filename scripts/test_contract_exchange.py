#!/usr/bin/env python3
"""Pinned, local contract exchange. Test data only; no device, API or rollout."""
import argparse
import copy
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from unittest import mock
import zipfile

from jsonschema import Draft202012Validator, FormatChecker
from referencing import Registry, Resource

ROOT = Path(__file__).resolve().parents[1]
PINS = {
    "core": "afe1a831b950a4e6560f0285a0103fc4eaa343b7",
    "console": "a0ef857cf425c1bb14ad8f62acec21f8c1324866",
    "registry": "0268b3748042d04fd3561553616388368881e04a",
    "ri": "161318d2897d07d5628ef502cafba1c7c489fff0",
}
PASSIVE = "passive-report.v0.2.schema.json"
PROFILE = "calibration-profile.v0.1.schema.json"
POSITION = "position-observation.v0.1.schema.json"
ASSOCIATION = "ri-calibration-association.v0.1-candidate.schema.json"
IDENTITY = "canonical-identity.v0.1-candidate.schema.json"
RI_SCHEMA = "consumer-observation-ref.v0.1-candidate.schema.json"
CONSOLE_SCHEMA = "calibration-inspection.v0.4b.schema.json"


def check_pin(directory, key):
    head = subprocess.check_output(
        ["git", "-C", str(directory), "rev-parse", "HEAD"], text=True
    ).strip()
    assert head == PINS[key], (key, head)
    assert not subprocess.check_output(
        ["git", "-C", str(directory), "status", "--porcelain", "--untracked-files=no"],
        text=True,
    ).strip(), f"modified pinned {key} checkout"


def sha256(data):
    return "sha256:" + hashlib.sha256(data).hexdigest()


def read_schema(path):
    schema = json.loads(path.read_text(encoding="utf-8"))
    Draft202012Validator.check_schema(schema)
    return schema


def validators(paths):
    schemas = {name: read_schema(path) for name, path in paths.items()}
    # No remote retrieval function. All cross-references resolve through
    # exact local schema IDs, including populated CAL Measurement arrays.
    registry = Registry().with_resources(
        (schema["$id"], Resource.from_contents(schema))
        for schema in schemas.values()
    )
    return {
        key: Draft202012Validator(
            schema, registry=registry, format_checker=FormatChecker()
        )
        for key, schema in schemas.items()
    }


def rejected(validator, value, label):
    assert not validator.is_valid(value), f"schema accepted {label}"


def run_json(argv, **kwargs):
    result = subprocess.run(argv, capture_output=True, timeout=90, check=False, **kwargs)
    assert result.returncode == 0, result.stderr.decode("utf-8", "replace")
    return json.loads(result.stdout)


def real_ri_refs(ri_root, temp):
    sys.path.insert(0, str(ri_root / "src"))
    from reality_ingress.consumer_ref import (
        unverified_ref_from_envelope, verified_ref_from_store,
    )
    from reality_ingress.evidence import EvidenceStore, IntegrityError
    from reality_ingress.generic import SAMPLE_SCHEMA, envelope_from_sample
    from reality_ingress.timebase import receive_stamp

    store = EvidenceStore(temp / "ri_evidence")
    raw = b"synthetic camera bytes, not a physical measurement"
    env = store.record(
        data=raw, source_id="source.synthetic.camera", modality="rgb",
        reference_frame_id="frame.camera.synthetic", sequence=1,
        timestamp=receive_stamp(), media_type="image/jpeg",
        representation={"kind_str": "reencoded_jpeg_synthetic"},
    )
    original = copy.deepcopy(env)
    verified = verified_ref_from_store(store, env["observation_id_str"])
    assert env == original
    assert verified["payload_sha256_hex"] == hashlib.sha256(raw).hexdigest()
    assert verified["observation_id"] == env["observation_id_str"]
    assert verified["integrity_status"] == "verified_local_hash"
    unverified = unverified_ref_from_envelope(env)
    assert unverified["integrity_status"] == "not_verified"

    generic_sample = {
        "schema_ref_str": SAMPLE_SCHEMA,
        "producer_str": "synthetic.cal_contract_test",
        "source": {
            "source_id_str": "source.synthetic.imu",
            "modality_str": "joint_state",
            "reference_frame_id_str": "frame.synthetic.arm",
            "sample_sequence_uint": 7,
        },
        "temporal_binding": receive_stamp(),
        "payload": {"measurements": {}},
    }
    generic = envelope_from_sample(generic_sample, "0" * 64, "payloads/00/test.bin", 14)
    generic_ref = unverified_ref_from_envelope(generic)
    assert generic_ref["observation_id"] == generic["observation_id_str"]
    # The corruption test uses the RI-owned actual store, not a fake flag.
    store._payload_path(verified["payload_sha256_hex"]).write_bytes(b"tampered")
    try:
        verified_ref_from_store(store, env["observation_id_str"])
    except IntegrityError:
        pass
    else:
        raise AssertionError("RI returned hash-verified ref for corrupt bytes")
    return {"verified_camera": verified, "unverified_camera": unverified,
            "unverified_generic": generic_ref}


def validate_cal_exchange(probe, ri, v, temp):
    valid = real_ri_refs(ri, temp)
    base = valid["verified_camera"]
    # Synthetic boundary mutations of real RI-produced references, NOT new
    # producer attestations. JSON Schema lengths are Unicode code points.
    unicode_ref = copy.deepcopy(base)
    unicode_ref["source_id"] = "é" * 256
    unicode_ref["time"]["boot_id"] = "é" * 128
    valid["unicode_boundary"] = unicode_ref
    max_seq = copy.deepcopy(base)
    max_seq["sample_sequence"] = 2**64 - 1
    valid["u64_max"] = max_seq
    invalid = {}
    for field in ("source_verified", "robot_binding_verified", "history_committed", "actuation_enabled"):
        invalid[field] = dict(base, **{field: True})
    invalid["bad_observation_id"] = dict(base, observation_id="obs_wrong")
    invalid["bad_hash"] = dict(base, payload_sha256_hex="x" * 64)
    invalid["negative_sequence"] = dict(base, sample_sequence=-1)
    invalid["unknown_schema"] = dict(base, schema_version="unknown")
    invalid["oversized_source"] = dict(base, source_id="é" * 257)
    invalid["extra_payload_path"] = dict(base, payload_path="/never/access")
    invalid["forged_time"] = dict(base, time=dict(base["time"], source_measurement_time="2026-11-26"))
    invalid["forged_anchor"] = dict(base, time=dict(base["time"], chronograf_anchor_ref="not-real"))
    invalid["oversized_boot"] = dict(base, time=dict(base["time"], boot_id="é" * 129))
    # RI v0.1 has no integer maximum; CAL deliberately fails closed beyond
    # its u64 representation. Owner decision, not a silent schema rewrite.
    overflow = dict(base, sample_sequence=2**64)
    v[RI_SCHEMA].validate(overflow)

    for name, ref in valid.items():
        v[RI_SCHEMA].validate(ref)
    for name, ref in invalid.items():
        rejected(v[RI_SCHEMA], ref, name)
    refs = {**valid, **invalid, "u64_overflow": overflow}
    out = run_json([str(probe)], input=json.dumps(refs, ensure_ascii=False).encode("utf-8"))

    for key in ("profile", "empty_profile"):
        v[PROFILE].validate(out[key])
    v[POSITION].validate(out["unknown_position"])
    for key in ("passive", "with_profile"):
        v[PASSIVE].validate(out[key])
        assert out[key]["claims_physical_calibration_certified"] is False
    assert out["with_profile"]["export_preview"]["applied"] is False
    assert out["with_profile"]["export_preview"]["hardware_binding_verified"] is False
    assert any(row["stored_measurements"] for row in out["with_profile"]["parameters"])
    assert all(out["runtime_rejections"].values()), out["runtime_rejections"]
    # Verify the real measurement cross-reference detects negative uncertainty.
    invalid_report = copy.deepcopy(out["with_profile"])
    row = next(r for r in invalid_report["parameters"] if r["stored_measurements"])
    row["stored_measurements"][0]["standard_uncertainty"] = -0.1
    rejected(v[PASSIVE], invalid_report, "negative uncertainty via Measurement ref")
    invalid_profile = copy.deepcopy(out["profile"])
    invalid_profile["measurements"][0]["evidence_ids"].append("synthetic_claim")
    rejected(v[PROFILE], invalid_profile, "duplicate evidence reference")
    rejected(v[POSITION], dict(out["unknown_position"], reference_kind="verified_forever"), "position reference enum")
    # JSON schema is shape, runtime seal/evidence/lifecycle rules are separate.
    active = dict(out["profile"], mode="active")
    v[PROFILE].validate(active)
    assert out["runtime_rejections"]["active_mode"]

    sidecar_count = 0
    for name, ref in valid.items():
        result = out["ri"][name]
        assert result["accepted"] is True, name
        for kind in ("experimental", "missing", "sealed", "expired"):
            sidecar = result[kind]
            v[ASSOCIATION].validate(sidecar)
            # CAL's schema intentionally does not own the nested RI shape.
            v[RI_SCHEMA].validate(sidecar["observation_ref"])
            assert sidecar["observation_ref"] == ref
            for flag in ("source_verified", "robot_binding_verified", "metrology_certified",
                         "history_committed", "actuation_enabled"):
                assert sidecar["assurance"][flag] is False
            assert sidecar["depth_consumer"]["metric_depth_allowed"] is False
            assert sidecar["depth_consumer"]["output_unit"] == "unknown"
            sidecar_count += 1
        assert result["experimental"]["artifact_resolution_status"] == "unverified_claim"
        assert result["sealed"]["artifact_resolution_status"] == "cal_profile_seal_only"
        assert result["expired"]["calibration_profile_state"] == "expired"
        assert result["missing"]["artifact_resolution_status"] == "missing"
    for name in (*invalid, "u64_overflow"):
        assert out["ri"][name] == {"accepted": False}, name
    forged = copy.deepcopy(out["ri"]["verified_camera"]["experimental"])
    forged["depth_consumer"]["metric_depth_allowed"] = True
    rejected(v[ASSOCIATION], forged, "metric depth authority")
    print(f"PASS CAL schemas: 2 Rust profiles, 1 position, 2 passive reports; populated Measurement reference resolved locally; 5 runtime rejection checks")
    print(f"PASS RI exchange: {len(valid)} accepted inputs ({3} actual producer projections), {sidecar_count} real Rust sidecars, {len(invalid) + 1} rejected inputs; RI store corruption rejected")
    print("GAP RI: integer domain is unbounded in schema vs CAL u64; owner review required. Shape-valid active CAL mode remains runtime-disabled.")


def zip_bytes(core):
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as archive:
        for file in sorted((core / "spec").rglob("*")):
            if file.is_file():
                archive.write(file, file.relative_to(core).as_posix())
    return buffer.getvalue()


def validate_core_console(core, console, registry, binary, v):
    sidecar = run_json([
        sys.executable, "-I", "-B", str(ROOT / "tools/canonical_identity/verify.py"),
        "--bundle", str(core), "--core-checkout", str(core), "--cal-binary", str(binary),
    ])
    v[IDENTITY].validate(sidecar)
    assert sidecar["integrity_status"] == "source_tree_hash_match"
    assert sidecar["report_binding_status"] == "source_index_match"
    assert sidecar["bundle_completeness"] == "not_assessed"
    assert sidecar["profile_model_hash_bound"] is False
    v[PASSIVE].validate(sidecar["calibration_report"])

    sys.path.insert(0, str(registry))
    sys.path.insert(0, str(console / "services/module-agent"))
    import calibration_passive as agent

    blob = zip_bytes(core)
    digest = sha256(binary.read_bytes())
    # Exercise the actual pinned Module Agent class and real CAL executable.
    # Only installation status and resolution are injected; no install/build
    # is requested via the agent. This is not a real installed Registry proof.
    with mock.patch.object(agent.registry, "status_of", return_value={"status": "ready"}), \
         mock.patch.object(agent.registry, "resolve_binary", return_value={
             "binary": str(binary), "binary_sha256": digest,
         }):
        with tempfile.TemporaryDirectory(prefix="cal-contract-agent-") as prefix:
            inspection = agent.CalibrationInspector(Path(prefix)).inspect(blob)
    v[PASSIVE].validate(inspection["report"])
    assert inspection["report"]["source_digests"] == sidecar["calibration_report"]["source_digests"]
    assert inspection["report"]["coverage"] == sidecar["calibration_report"]["coverage"]
    # Assemble a synthetic API wrapper using routes.rs field semantics;
    # the actual Rust Console API server is NOT executed in this job.
    envelope = {
        "schema_version": "console.calibration-inspection.v0.4b",
        "robot_id": "synthetic_robot",
        "source_model_id": "synthetic_console_model",
        "source_profile_sha256": sha256(blob),
        "source_integrity": "canonical_not_verified",
        "generated_at": inspection["generated_at"],
        "evidence_class": "declared_profile_only_not_physical_observation",
        "physical_calibration_certified": False,
        "actuation_enabled": False,
        "inspection": inspection,
    }
    v[CONSOLE_SCHEMA].validate(envelope)
    assert envelope["source_profile_sha256"] != sidecar["computed_source_tree_sha256"]
    rejected(v[CONSOLE_SCHEMA], dict(envelope, source_integrity="canonical_verified"), "legacy wrapper canonical upgrade")
    for flag in ("claims_physical_calibration_certified", "current_robot_position_observed", "physical_drift_evaluated"):
        changed = copy.deepcopy(inspection["report"])
        changed[flag] = True
        rejected(v[PASSIVE], changed, flag)
        try:
            agent.validate_cal02_report(changed)
        except agent.InspectionError:
            pass
        else:
            raise AssertionError(f"Console accepted authority {flag}")

    # Characterization of existing consumer limitations; never label as PASS
    # of full Console schema conformance. Record exact drift for owner.
    for name, bad in (
        ("missing_export_preview", {k: x for k, x in inspection["report"].items() if k != "export_preview"}),
        ("fabricated_profile_binding", dict(inspection["report"], profile_binding="claimed_match", profile_id="fake")),
        ("bad_source_digest", dict(inspection["report"], source_digests=[{"role": "description", "path": "invalid", "raw_file_sha256": "bad"}])),
    ):
        if name != "fabricated_profile_binding":
            rejected(v[PASSIVE], bad, name)
        # The passive schema permits historical profiles; Console's no-profile
        # route must additionally enforce that context. Existing agent does not.
        assert agent.validate_cal02_report(bad) == bad, name
        print(f"GAP Console v0.4.3 agent accepts {name}; preserve owner-side remediation gate")
    print("PASS Core/Console: actual Core identity sidecar + nested CAL report; actual Console extractor and Agent subprocess with injected Registry resolution; synthetic API wrapper validates. ZIP hash remains distinct.")


def main():
    parser = argparse.ArgumentParser()
    for key in PINS:
        parser.add_argument("--" + key, type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--probe", type=Path, required=True)
    args = parser.parse_args()
    pinned = {key: getattr(args, key).resolve(strict=True) for key in PINS}
    for key, directory in pinned.items():
        check_pin(directory, key)
    paths = {name: ROOT / "contracts" / name for name in (PASSIVE, PROFILE, POSITION, ASSOCIATION, IDENTITY)}
    paths[RI_SCHEMA] = pinned["ri"] / "contracts" / RI_SCHEMA
    paths[CONSOLE_SCHEMA] = pinned["console"] / "contracts/console/v1" / CONSOLE_SCHEMA
    v = validators(paths)
    # The new RI stack retains the exact consumer schema/producer pinned by CAL-RI-01.
    for path, expected in (
        ("contracts/" + RI_SCHEMA, "3d92e074e71593bf6db0a735394c628ea131668d"),
        ("src/reality_ingress/consumer_ref.py", "a34f4cbd924b58c849af215d175500f5a5ca2ece"),
    ):
        blob = subprocess.check_output(["git", "-C", str(pinned["ri"]), "hash-object", path], text=True).strip()
        assert blob == expected, (path, blob)
    with tempfile.TemporaryDirectory(prefix="cal-021-c-") as tmp:
        validate_cal_exchange(args.probe.resolve(strict=True), pinned["ri"], v, Path(tmp))
        validate_core_console(pinned["core"], pinned["console"], pinned["registry"], args.binary.resolve(strict=True), v)
    print("PASS cross-repository candidate suite. No physical evidence, deployed Console, RI session resolver approval, Registry installation, or runtime authority.")


if __name__ == "__main__":
    main()
