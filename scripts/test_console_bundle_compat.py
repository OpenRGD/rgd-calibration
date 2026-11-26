#!/usr/bin/env python3
"""CAL-021-A: exercise the real pinned Console Module Agent, with the candidate
CAL binary, on private synthetic ZIPs. No registry installation, network during
tests, Console service, hardware, or robot access.
"""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import warnings
import zipfile


ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / "tests/fixtures/cal-021-a/bundle-syntax-corpus.v1.json"
DESCRIPTION = "spec/01_foundation/description.jsonc"
MODEL = b'{"meta_group":{"model_id_str":"cal021_console_synthetic"},"joint_topology_map":{},"sensors_mounting_map":{}}'


def archive(entries):
    buffer = io.BytesIO()
    with warnings.catch_warnings(), zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as zf:
        warnings.simplefilter("ignore", UserWarning)
        for name, payload in entries:
            zf.writestr(name, payload)
    return buffer.getvalue()


def execute(agent, binary, entries, accepted=True):
    with tempfile.TemporaryDirectory(prefix="cal021-interop-") as tmp:
        source = agent.extract_foundation_zip(archive(entries), Path(tmp))
        result = subprocess.run(
            [str(binary), "inspect", "--bundle", str(source.root)],
            capture_output=True,
            timeout=35,
            check=False,
            env={"PATH": os.defpath, "LANG": "C.UTF-8"},
        )
        if not accepted:
            assert result.returncode == 2, result.stderr
            assert b"CAL_BUNDLE_SYNTAX" in result.stderr, result.stderr
            assert not result.stdout, "failure must not emit a successful report"
            return None
        assert result.returncode == 0, result.stderr
        report = agent.validate_cal02_report(json.loads(result.stdout))
        assert report["report_version"] == "0.2.0"
        assert report["profile_id"] is None
        assert report["profile_binding"] == "no_profile"
        assert report["export_preview"] is None
        assert report["claims_physical_calibration_certified"] is False
        assert report["current_robot_position_observed"] is False
        assert report["physical_drift_evaluated"] is False
        # The real extractor must preserve bytes. CAL hashes those bytes, not
        # normalized JSON or the ZIP. No canonical integrity claim is inferred.
        expected = {
            name.rsplit("/", 1)[-1]: payload
            for name, payload in entries
            if isinstance(name, str) and name.rsplit("/", 1)[-1] in source.extracted
            and name.endswith(".jsonc")
        }
        assert len(report["source_digests"]) == len(source.extracted)
        for entry in report["source_digests"]:
            filename = entry["path"].rsplit("/", 1)[-1]
            raw = expected[filename]
            assert (source.root / entry["path"]).read_bytes() == raw
            assert entry["raw_file_sha256"] == "sha256:" + hashlib.sha256(raw).hexdigest()
        return report


def expect_extract_rejected(agent, entries):
    with tempfile.TemporaryDirectory(prefix="cal021-bad-zip-") as tmp:
        try:
            agent.extract_foundation_zip(archive(entries), Path(tmp))
        except agent.InspectionError:
            return
        raise AssertionError("Console extraction accepted a rejected ZIP case")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--console", type=Path, required=True)
    parser.add_argument("--registry", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    sys.path.insert(0, str(args.registry.resolve(strict=True)))
    sys.path.insert(0, str(args.console.resolve(strict=True) / "services/module-agent"))
    import calibration_passive as agent

    assert agent.MODULE_ID == "org.openrgd.calibration"
    assert agent.MODULE_VERSION == "0.2.0"

    cases = json.loads(CORPUS.read_text(encoding="utf-8"))["cases"]
    count = 0
    for case in cases:
        # Current Console ZIP contract accepts source .jsonc only. The direct
        # CAL .json cases run independently in Rust. No fake file conversion.
        if case["syntax"] != "jsonc":
            continue
        execute(agent, binary, [(DESCRIPTION, case["raw"].encode("utf-8"))], case["accepted"])
        count += 1

    partial = execute(agent, binary, [("robot/" + DESCRIPTION, MODEL)])
    assert partial["coverage"]["candidate_fields"] == 0
    assert len(partial["source_digests"]) == 1
    assert sum(f["code"] == "optional_source_not_provided" for f in partial["findings"]) == 4

    full = [(DESCRIPTION, MODEL)]
    for role in ("calibration", "actuation_topology", "hal_mapping", "sensor_fidelity"):
        full.append(("spec/01_foundation/" + role + ".jsonc", b"{}"))
    full.extend([
        ("spec/manifest.jsonc", b"{}"),
        ("spec/00_core/kernel.jsonc", b"{}"),
        ("standard/01_foundation/description.json", b'{"ignored":"mirror"}'),
    ])
    report = execute(agent, binary, full)
    assert len(report["source_digests"]) == 5
    assert not any(f["code"] == "optional_source_not_provided" for f in report["findings"])
    # Ambiguous roots and unsafe entries are rejected by the *actual* agent,
    # independently of CAL's directory parser.
    link = zipfile.ZipInfo("spec/01_foundation/calibration.jsonc")
    link.create_system = 3
    link.external_attr = (stat.S_IFLNK | 0o777) << 16
    bad = [
        [("standard/01_foundation/description.json", MODEL)],
        [(DESCRIPTION, MODEL), ("other/" + DESCRIPTION, MODEL)],
        [(DESCRIPTION, MODEL), ("../escape.jsonc", b"{}")],
        [(DESCRIPTION, MODEL), (DESCRIPTION, MODEL)],
        [(DESCRIPTION, MODEL), (link, b"/tmp/untrusted")],
        [(DESCRIPTION, MODEL), ("spec/01_foundation/calibration.jsonc", b" " * (2 * 1024 * 1024 + 1))],
    ]
    for entries in bad:
        expect_extract_rejected(agent, entries)

    print(f"PASS: {count} shared JSONC cases, 2 bundle-layout checks and {len(bad)} ZIP negative checks against real Console v0.4.3 extractor/validator")
    print("No registry installation, API/UI, canonical hash verification, camera or motor access.")


if __name__ == "__main__":
    main()
