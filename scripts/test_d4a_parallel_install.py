#!/usr/bin/env python3
"""D.4A actual Registry install/resolve with a private two-version catalog.

No source/build mocks, no production descriptor or catalog edits, no host
activation. Network is used by the real installer during CI setup/install.
The post-install process tests lack Git/Cargo on PATH; this is not a network
namespace or a deployed host offline/reboot test.
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

from jsonschema import Draft202012Validator, FormatChecker
from referencing import Registry, Resource

ROOT = Path(__file__).resolve().parents[1]
MODULE = "org.openrgd.calibration"
OLD = "0.2.0"
NEW = "0.2.1"
REGISTRY_PIN = "0268b3748042d04fd3561553616388368881e04a"
OLD_PIN = "81700401e2fa6d370a86b6c1f5a48802bc5f1fc7"
NEW_PIN = "292d42aab5e04392b736693233811734bffb603e"
D3_MERGE = "f912a961f55bb051022eadc25e1ebd8b95e8f4ef"
PASSIVE = "contracts/passive-report.v0.2.schema.json"
PROFILE = "contracts/calibration-profile.v0.1.schema.json"
BINARY = "rgd-calibration"


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def digest(path):
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def tree_hashes(path):
    return {p.relative_to(path).as_posix(): digest(p)
            for p in sorted(path.rglob("*")) if p.is_file()}


def git(path, *args):
    return subprocess.check_output(["git", "-C", str(path), *args], text=True).strip()


def run(argv, *, cwd, env=None, expected=0, timeout=600):
    r = subprocess.run([str(x) for x in argv], cwd=cwd, env=env,
                       capture_output=True, timeout=timeout, check=False)
    assert r.returncode == expected, (argv[:5], r.returncode, r.stderr[-1800:])
    if expected:
        assert not r.stdout, ("unexpected success output", argv[:5])
        assert r.stderr
    return r


def registry_cli(work, *args, env=None, expected=0):
    r = run([sys.executable, "-B", "-m", "rgd_registry", *args],
            cwd=work, env=env, expected=expected, timeout=1500)
    return json.loads(r.stdout) if expected == 0 else r.stderr.decode("utf-8", "replace")


def lookup(work, verb, prefix, version, *, env=None, expected=0):
    args = [verb, MODULE, "--prefix", str(prefix)]
    if version is not None:
        args += ["--version", version]
    return registry_cli(work, *args, env=env, expected=expected)


def cli_env():
    env = os.environ.copy()
    for key in ("PYTHONPATH", "PYTHONHOME"):
        env.pop(key, None)
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    # Registry setup invokes Git/Cargo explicitly; do not inherit a user's
    # alternate Cargo target path which the bootstrap installer cannot resolve.
    env.pop("CARGO_TARGET_DIR", None)
    return env


def private_catalog(registry_checkout, accepted_checkout, work, env):
    assert git(registry_checkout, "rev-parse", "HEAD") == REGISTRY_PIN
    assert git(accepted_checkout, "rev-parse", "HEAD") == NEW_PIN
    for root in (registry_checkout, accepted_checkout):
        assert git(root, "status", "--porcelain", "--untracked-files=no") == ""
    shutil.copytree(registry_checkout / "rgd_registry", work / "rgd_registry")
    shutil.copytree(registry_checkout / "registry", work / "registry")
    original = work / "registry/modules" / MODULE / (OLD + ".json")
    old_bytes = original.read_bytes()
    old = read_json(original)
    assert old["source"]["commit_sha"] == OLD_PIN
    assert old["permissions_required"] == []
    assert old["safety"] == {"non_actuating": True, "automatic_motion_on_load": False,
                             "active_calibration_supported": False}
    assert [row["version"] for row in registry_cli(work, "catalog", env=env)["modules"]] == [OLD]
    new = json.loads(json.dumps(old))
    new["version"] = NEW
    new["source"]["commit_sha"] = NEW_PIN
    new["integration"]["argument_contract"] = "docs/CLI_CONTRACT.md"
    new["integration"]["report_contract"] = (
        "https://github.com/OpenRGD/rgd-calibration/blob/" + NEW_PIN + "/" + PASSIVE
    )
    root = work / "registry"
    module_schema = read_json(root / "schemas/module.v0.1.schema.json")
    index_schema = read_json(root / "schemas/index.v0.1.schema.json")
    for schema in (module_schema, index_schema):
        Draft202012Validator.check_schema(schema)
    Draft202012Validator(module_schema).validate(new)
    write_json(root / "modules" / MODULE / (NEW + ".json"), new)
    index = read_json(root / "index.json")
    index["modules"].append({"id": MODULE, "version": NEW,
                             "path": "modules/" + MODULE + "/" + NEW + ".json"})
    Draft202012Validator(index_schema).validate(index)
    write_json(root / "index.json", index)
    assert original.read_bytes() == old_bytes
    assert (registry_checkout / "registry/modules" / MODULE / (OLD + ".json")).read_bytes() == old_bytes
    assert not (registry_checkout / "registry/modules" / MODULE / (NEW + ".json")).exists()
    assert registry_cli(work, "catalog", env=env)["modules"] == [old, new]
    # Actual CLI must fail on implicit latest, not choose one of two versions.
    for verb in ("show", "status", "resolve", "install"):
        message = registry_cli(work, verb, MODULE, env=env, expected=2)
        assert "version required" in message
    print("PASS private catalog: actual registered 0.2.0 descriptor unchanged, exact accepted 0.2.1 source, explicit versions required", flush=True)


def install(work, prefix, version, env):
    result = registry_cli(work, "install", MODULE, "--version", version,
                          "--prefix", str(prefix), "--build", env=env)
    assert result["status"] == "ready" and result["version"] == version
    expected_pin = OLD_PIN if version == OLD else NEW_PIN
    assert result["source_commit"] == expected_pin
    install_root = prefix / "modules" / MODULE / version
    binary = install_root / "bin" / BINARY
    assert Path(result["binary"]) == binary
    assert digest(binary) == result["binary_sha256"]
    receipt = read_json(install_root / "installed.json")
    assert receipt["source_commit"] == expected_pin
    assert receipt["binary_sha256"] == result["binary_sha256"]
    assert receipt["source_tree_sha256"].startswith("sha256:")
    assert not (install_root / "source/.git").exists()
    assert not (install_root / "source/target").exists()
    print(f"INSTALLED {version} source={expected_pin} binary={result['binary_sha256']}", flush=True)
    return result, install_root, binary


def validator(checkout):
    schemas = [read_json(checkout / p) for p in (PASSIVE, PROFILE)]
    for schema in schemas:
        Draft202012Validator.check_schema(schema)
    resources = Registry().with_resources(
        (s["$id"], Resource.from_contents(s)) for s in schemas
    )
    return Draft202012Validator(schemas[0], registry=resources, format_checker=FormatChecker())


def inspect(binary, bundle, env, folder, v, profile=None, expected=0, validate_schema=True):
    args = [binary, "inspect", "--bundle", bundle]
    if profile is not None:
        args.extend(["--profile", profile])
    r = run(args, cwd=folder, env=env, expected=expected, timeout=35)
    if expected:
        return None
    report = json.loads(r.stdout)
    if validate_schema:
        v.validate(report)
    assert report["report_version"] == "0.2.0"
    for key in ("current_robot_position_observed", "physical_drift_evaluated",
                "claims_physical_calibration_certified"):
        assert report[key] is False
    assert report["export_preview"] is None
    return report


def check_historical_profiles(old_source, old_bin, new_bin, bundle, temp, env, local_env, v):
    # Compile a test-only generator against a PRIVATE copy of the actual
    # legacy installed Rust source. Never mutate its seal or installed tree.
    # Dependencies are already cached by the explicit legacy installation.
    fixture_checkout = temp / "legacy-fixture-source"
    shutil.copytree(old_source, fixture_checkout)
    example = fixture_checkout / "examples/d4_legacy_profile_fixture.rs"
    example.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(ROOT / "tests/support/d4_legacy_profile_fixture.rs", example)
    result = run(["cargo", "run", "--quiet", "--offline", "--locked",
                  "--manifest-path", fixture_checkout / "Cargo.toml",
                  "--example", "d4_legacy_profile_fixture"],
                 cwd=fixture_checkout, env=env, timeout=600)
    records = json.loads(result.stdout)
    assert set(records) == {"good", "duplicate", "dangling"}
    stored = temp / "historical-profiles"
    for name, profile in records.items():
        assert profile["schema_version"] == "0.1.0"
        assert profile["integrity_sha256"].startswith("sha256:")
        assert profile["state"] == "uncalibrated"
        file = stored / (name + ".json")
        write_json(file, profile)
        before = file.read_bytes()
        legacy = inspect(old_bin, bundle, local_env, temp, v, profile=file,
                         validate_schema=name == "good")
        assert legacy["profile_id"] == profile["profile_id"]
        # The legacy report containing duplicate evidence IDs can violate the
        # newer schema's uniqueItems rule. Its actual acceptance is preserved
        # by the actual legacy CLI invocation, not as new schema validity.
        assert before == file.read_bytes()
        if name == "good":
            new = inspect(new_bin, bundle, local_env, temp, v, profile=file)
            assert new["profile_id"] == legacy["profile_id"]
        else:
            inspect(new_bin, bundle, local_env, temp, v, profile=file, expected=2)
        assert file.read_bytes() == before
    return tree_hashes(stored)


def exercise(registry_checkout, accepted_checkout, temp, evidence_out):
    env = cli_env()
    work = temp / "private-registry"
    private_catalog(registry_checkout, accepted_checkout, work, env)
    prefix = temp / "isolated-prefix"
    old, old_root, old_bin = install(work, prefix, OLD, env)
    old_before = tree_hashes(old_root)
    new, new_root, new_bin = install(work, prefix, NEW, env)
    new_before = tree_hashes(new_root)
    assert old_root != new_root and old_bin != new_bin
    assert tree_hashes(old_root) == old_before
    for name in ("Cargo.toml", "Cargo.lock", "module.openrgd.json",
                 PASSIVE, PROFILE, "src/bin/rgd-calibration.rs"):
        assert (new_root / "source" / name).read_bytes() == (accepted_checkout / name).read_bytes()
    # New interpreter processes, no Git or Cargo on PATH for lookup/inspect.
    empty_path = temp / "no-external-tools"
    empty_path.mkdir()
    local_env = dict(env, PATH=str(empty_path))
    assert shutil.which("git", path=local_env["PATH"]) is None
    assert shutil.which("cargo", path=local_env["PATH"]) is None
    for version, receipt in ((OLD, old), (NEW, new)):
        ready = lookup(work, "status", prefix, version, env=local_env)
        resolved = lookup(work, "resolve", prefix, version, env=local_env)
        assert ready == resolved == receipt
    assert run([new_bin, "--version"], cwd=temp, env=local_env).stdout == b"rgd-calibration 0.2.1\n"
    run([old_bin, "--version"], cwd=temp, env=local_env, expected=2)
    v = validator(accepted_checkout)
    bundle = accepted_checkout / "tests/fixtures/passive-minimal"
    before_bundle = tree_hashes(bundle)
    old_report = inspect(old_bin, bundle, local_env, temp, v)
    new_report = inspect(new_bin, bundle, local_env, temp, v)
    for key in ("report_version", "model_id", "coverage", "source_digests"):
        assert old_report[key] == new_report[key], key
    # Historical report is stored as-is, not rebranded as a 0.2.1 preview.
    archive = temp / "historic-report.json"
    write_json(archive, old_report)
    old_report_bytes = archive.read_bytes()
    for version in (OLD, NEW):
        msg = registry_cli(work, "install", MODULE, "--version", version,
                           "--prefix", str(prefix), "--build", env=local_env, expected=2)
        assert "already installed" in msg
    assert tree_hashes(old_root) == old_before
    assert tree_hashes(new_root) == new_before
    profiles = check_historical_profiles(old_root / "source", old_bin, new_bin,
                                         bundle, temp, env, local_env, v)
    assert tree_hashes(old_root) == old_before and tree_hashes(new_root) == new_before
    print("PASS legacy profiles: historical seal created by old library, compatible record accepted, duplicate/dangling evidence rejected by new validator; originals unchanged", flush=True)
    # Deliberate disposable tamper. Never repair or silently downgrade.
    new_bin.write_bytes(b"SYNTHETIC_D4_TAMPER")
    assert lookup(work, "status", prefix, NEW, env=local_env)["status"] == "tampered"
    lookup(work, "resolve", prefix, NEW, env=local_env, expected=2)
    lookup(work, "resolve", prefix, None, env=local_env, expected=2)
    rollback = lookup(work, "resolve", prefix, OLD, env=local_env)
    assert rollback == old
    assert inspect(old_bin, bundle, local_env, temp, v) == old_report
    assert tree_hashes(old_root) == old_before
    assert tree_hashes(bundle) == before_bundle
    assert archive.read_bytes() == old_report_bytes
    assert tree_hashes(temp / "historical-profiles") == profiles
    print("PASS actual isolated parallel install, fresh resolution without Git/Cargo, explicit legacy rollback after new-binary tamper; original installation/profile/report/bundle unchanged", flush=True)
    # Record exact hashes of the actual installed binaries, not D.3 CI hash
    # as an install receipt. The private descriptor/catalog are not exported.
    evidence = {
        "schema_version": "cal.d4-install-review.v0.1-candidate",
        "scope": "isolated_ci_real_registry_install_not_publication",
        "registry_commit": REGISTRY_PIN,
        "package_acceptance_merge": D3_MERGE,
        "versions": [
            {"version": item["version"], "source_commit": item["source_commit"],
             "binary_sha256": item["binary_sha256"]}
            for item in (old, new)
        ],
        "historic_profile_cases": 3,
        "rollback_selected_version": OLD,
        "external_build_tools_absent_on_resolution_path": True,
        "network_isolation_proven": False,
        "console_version_aware_consumer_tested": False,
        "publication_authorized": False,
        "physical_calibration_certified": False,
        "actuation_enabled": False,
    }
    write_json(evidence_out, evidence)
    print("PASS D.4A scoped CI only; Console/Registry owner integration, migration decisions and real host E/F remain open", flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--registry", type=Path, required=True)
    parser.add_argument("--accepted", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="cal-d4a-") as tmp:
        exercise(args.registry.resolve(strict=True), args.accepted.resolve(strict=True),
                 Path(tmp), args.evidence.resolve())


if __name__ == "__main__":
    main()
