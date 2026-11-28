#!/usr/bin/env python3
"""D.3 package QA. Actual release-mode executable; no Registry publication.

The staged artifact is a CI review artifact only, not a distribution entry.
Hashes record this build, not reproducibility across hosts or signed provenance.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib

from jsonschema import Draft202012Validator, FormatChecker
from referencing import Registry, Resource

ROOT = Path(__file__).resolve().parents[1]
VERSION = "0.2.1"
MODULE = "org.openrgd.calibration"
REGISTRY_PIN = "0268b3748042d04fd3561553616388368881e04a"
LEGACY_PIN = "81700401e2fa6d370a86b6c1f5a48802bc5f1fc7"
BASE_PIN = "68f5b855e10d4549f639c5044d99951f6a61b78e"
PASSIVE = "contracts/passive-report.v0.2.schema.json"
PROFILE = "contracts/calibration-profile.v0.1.schema.json"
BINARY = "rgd-calibration"


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(path, *args):
    return subprocess.check_output(["git", "-C", str(path), *args], text=True).strip()


def execute(binary, *args, expected=0, cwd=None):
    result = subprocess.run([str(binary), *map(str, args)], capture_output=True,
                            timeout=35, check=False, cwd=cwd,
                            env={"PATH": os.defpath, "LANG": "C.UTF-8"})
    assert result.returncode == expected, (args, result.returncode, result.stderr[:1000])
    if expected:
        assert not result.stdout, (args, "failure emitted stdout")
        assert result.stderr, (args, "missing diagnostic")
    return result


def verify_source(registry_path, baseline):
    assert git(registry_path, "rev-parse", "HEAD") == REGISTRY_PIN
    assert git(baseline, "rev-parse", "HEAD") == BASE_PIN
    assert git(ROOT, "status", "--porcelain", "--untracked-files=no") == ""
    sys.path.insert(0, str(registry_path))
    from rgd_registry import core as registry
    descriptor = registry.entry(MODULE, "0.2.0")
    assert registry.catalog() == [descriptor]
    assert descriptor["source"]["commit_sha"] == LEGACY_PIN
    try:
        registry.entry(MODULE, VERSION)
    except registry.RegistryError:
        pass
    else:
        raise AssertionError("unexpected registered 0.2.1")
    candidate = json.loads(json.dumps(descriptor))
    candidate["version"] = VERSION
    candidate["source"]["commit_sha"] = git(ROOT, "rev-parse", "HEAD")
    candidate["integration"]["argument_contract"] = "docs/CLI_CONTRACT.md"
    candidate["integration"]["report_contract"] = (
        "https://github.com/OpenRGD/rgd-calibration/blob/" +
        candidate["source"]["commit_sha"] + "/" + PASSIVE
    )
    # This object never leaves memory or becomes a catalog entry.
    schema = read_json(registry_path / "registry/schemas/module.v0.1.schema.json")
    Draft202012Validator.check_schema(schema)
    Draft202012Validator(schema).validate(candidate)
    registry._verify_upstream_manifest(ROOT, candidate)
    registry._verify_upstream_manifest(baseline, descriptor)
    try:
        registry._verify_upstream_manifest(ROOT, descriptor)
    except registry.RegistryError:
        pass
    else:
        raise AssertionError("new package accepted as 0.2.0")
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
    old_cargo = tomllib.loads((baseline / "Cargo.toml").read_text())
    assert cargo["package"]["version"] == VERSION
    assert cargo["package"]["name"] == BINARY
    assert cargo["features"]["default"] == []
    assert cargo["features"] == old_cargo["features"]
    assert cargo["dependencies"] == old_cargo["dependencies"]
    assert cargo["example"] == old_cargo["example"]
    assert cargo["example"][0]["required-features"] == ["ri-candidate"]
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    old_lock = tomllib.loads((baseline / "Cargo.lock").read_text())
    package = [p for p in lock["package"] if p["name"] == BINARY]
    assert len(package) == 1 and package[0]["version"] == VERSION
    package[0]["version"] = "0.2.0"
    assert lock == old_lock, "dependency resolution changed"
    manifest = read_json(ROOT / "module.openrgd.json")
    old_manifest = read_json(baseline / "module.openrgd.json")
    assert manifest["module_version"] == VERSION
    assert manifest["registry_status"] == "proposal-only"
    manifest["module_version"] = "0.2.0"
    assert manifest == old_manifest, "manifest capability/safety change"
    # No report schema or Rust library change hidden in package preparation.
    for folder in ("src", "contracts"):
        paths = [p.relative_to(ROOT) for p in (ROOT / folder).rglob("*") if p.is_file()]
        old_paths = [p.relative_to(baseline) for p in (baseline / folder).rglob("*") if p.is_file()]
        assert sorted(paths) == sorted(old_paths), folder
        for p in paths:
            if p.as_posix() == "src/bin/rgd-calibration.rs":
                continue
            assert (ROOT / p).read_bytes() == (baseline / p).read_bytes(), str(p)
    print("PASS aligned 0.2.1 package/lock/manifest, actual Registry manifest checks, unchanged dependencies/capabilities/library/schemas; no registered 0.2.1")


def report_validator():
    passive = read_json(ROOT / PASSIVE)
    profile = read_json(ROOT / PROFILE)
    for schema in (passive, profile):
        Draft202012Validator.check_schema(schema)
    assert passive["properties"]["report_version"]["const"] == "0.2.0"
    assert profile["properties"]["schema_version"]["const"] == "0.1.0"
    resources = Registry().with_resources(
        (s["$id"], Resource.from_contents(s)) for s in (passive, profile)
    )
    return Draft202012Validator(passive, registry=resources, format_checker=FormatChecker())


def check_binary(binary, baseline_binary, validator):
    with tempfile.TemporaryDirectory(prefix="cal-d3-qa-") as tmp:
        folder = Path(tmp)
        version = execute(binary, "--version", cwd=folder)
        assert version.stdout == b"rgd-calibration 0.2.1\n" and not version.stderr
        assert not list(folder.iterdir())
        execute(baseline_binary, "--version", expected=2, cwd=folder)
        for name in ("passive-minimal", "openrgd-reference"):
            fixture = ROOT / "tests/fixtures" / name
            original_files = {p.relative_to(fixture): digest(p)
                              for p in fixture.rglob("*") if p.is_file()}
            result = execute(binary, "inspect", "--bundle", fixture, cwd=folder)
            baseline = execute(baseline_binary, "inspect", "--bundle", fixture, cwd=folder)
            assert result.stdout == baseline.stdout, name + ": report bytes changed"
            report = json.loads(result.stdout)
            validator.validate(report)
            for key in ("current_robot_position_observed", "physical_drift_evaluated",
                        "claims_physical_calibration_certified"):
                assert report[key] is False
            assert report["profile_binding"] == "no_profile"
            assert report["profile_id"] is None and report["export_preview"] is None
            assert original_files == {p.relative_to(fixture): digest(p)
                                      for p in fixture.rglob("*") if p.is_file()}
        assert not list(folder.iterdir())
        fixture = ROOT / "tests/fixtures/passive-minimal"
        saved = folder / "report.json"
        outcome = execute(binary, "inspect", "--bundle", fixture, "--out", saved, cwd=folder)
        assert not outcome.stdout
        validator.validate(read_json(saved))
        old_bytes = saved.read_bytes()
        execute(binary, "inspect", "--bundle", fixture, "--out", saved, expected=2, cwd=folder)
        assert saved.read_bytes() == old_bytes
        rejected = [
            (), ("--version", "--bundle", fixture), ("install",), ("calibrate",),
            ("move",), ("seal",), ("inspect",), ("inspect", "--bundle"),
            ("inspect", "--bundle", "--version"),
            ("inspect", "--bundle", fixture, "--bundle", fixture),
            ("inspect", "--bundle", fixture, "--unknown", "x"),
            ("inspect", "--bundle", fixture, "--model-hash", "sha256:bad"),
            ("inspect", "--bundle", folder / "missing"),
        ]
        for args in rejected:
            execute(binary, *args, expected=2, cwd=folder)
        # Exercise actual release binary's conservative parser. No conversion.
        corpus = read_json(ROOT / "tests/fixtures/cal-021-a/bundle-syntax-corpus.v1.json")["cases"]
        for case in corpus:
            source = folder / "corpus" / case["id"] / "spec/01_foundation"
            source.mkdir(parents=True)
            path = source / ("description." + case["syntax"])
            raw = case["raw"].encode("utf-8")
            path.write_bytes(raw)
            result = execute(binary, "inspect", "--bundle", source.parents[1],
                             expected=0 if case["accepted"] else 2, cwd=folder)
            if case["accepted"]:
                validator.validate(json.loads(result.stdout))
            assert path.read_bytes() == raw
        print(f"PASS real release CLI: exact 0.2.1 metadata, 2 byte-identical baseline reports, no-overwrite output, {len(rejected)} invalid commands/options, {len(corpus)} syntax fixtures")


def stage(binary, out):
    out.mkdir(parents=True, exist_ok=False)
    # Only the passive executable and immutable build inputs/contracts, not
    # source tooling, examples, test binaries or a Registry descriptor.
    shutil.copy2(binary, out / BINARY)
    for filename in ("Cargo.toml", "Cargo.lock", "module.openrgd.json",
                     PASSIVE, PROFILE, "docs/CLI_CONTRACT.md", "LICENSE"):
        target = out / filename
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / filename, target)
    host = subprocess.check_output(["rustc", "-vV"], text=True).strip()
    assert "host: x86_64-unknown-linux-gnu" in host, host
    inputs = {p.relative_to(out).as_posix(): digest(p)
              for p in sorted(out.rglob("*")) if p.is_file()}
    provenance = {
        "schema_version": "cal.package-build-evidence.v0.1-candidate",
        "scope": "unsigned_ci_review_artifact_not_distribution",
        "module_id": MODULE,
        "package_version": VERSION,
        "report_version": "0.2.0",
        "profile_schema_version": "0.1.0",
        "source_commit": git(ROOT, "rev-parse", "HEAD"),
        "source_tree_git_id": git(ROOT, "rev-parse", "HEAD^{tree}"),
        "registry_reference_commit": REGISTRY_PIN,
        "legacy_registered_source_commit": LEGACY_PIN,
        "rustc": host,
        "cargo": subprocess.check_output(["cargo", "--version"], text=True).strip(),
        "build_command": ["cargo", "build", "--release", "--locked", "--bin", BINARY],
        "default_features": [],
        "ci_run_id": os.environ.get("GITHUB_RUN_ID"),
        "ci_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
        "files_sha256_hex": inputs,
        "publication_authorized": False,
        "signature_verified": False,
        "actuation_enabled": False,
    }
    (out / "BUILD-EVIDENCE.json").write_text(json.dumps(provenance, indent=2) + "\n")
    files = [p for p in sorted(out.rglob("*")) if p.is_file()]
    (out / "SHA256SUMS").write_text("".join(
        f"{digest(p)}  {p.relative_to(out).as_posix()}\n" for p in files))
    for name, value in inputs.items():
        assert digest(out / name) == value
    print("PACKAGE_SOURCE_COMMIT=" + provenance["source_commit"])
    print("PACKAGE_BINARY_SHA256=" + inputs[BINARY])
    print("PACKAGE_EVIDENCE_SHA256=" + digest(out / "BUILD-EVIDENCE.json"))
    print("PASS unsigned candidate review artifact; no Registry publication or runtime authority")


def main():
    parser = argparse.ArgumentParser()
    for name in ("registry", "baseline", "binary", "baseline-binary", "out"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    baseline = args.baseline.resolve(strict=True)
    verify_source(args.registry.resolve(strict=True), baseline)
    check_binary(args.binary.resolve(strict=True), args.baseline_binary.resolve(strict=True),
                 report_validator())
    stage(args.binary.resolve(strict=True), args.out.resolve())


if __name__ == "__main__":
    main()
