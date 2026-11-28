#!/usr/bin/env python3
"""D.2 engineering matrix and Registry behavior tests; no publication.

A/B/C/D.1 runtime evidence remains separately pinned. The two-version catalog
and installed trees created here are disposable synthetic fixtures. Actual
Registry code is exercised, but source fetch and Cargo execution are mocked.
"""
import argparse
import copy
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from unittest import mock

from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
MATRIX_DIR = ROOT / "docs/compatibility"
MODULE = "org.openrgd.calibration"
OLD = "0.2.0"
NEW = "0.2.1"
EXPECTED_PINS = {
    "registry": "0268b3748042d04fd3561553616388368881e04a",
    "console_released": "a0ef857cf425c1bb14ad8f62acec21f8c1324866",
    "console_candidate": "168e99c2ba057ae78bfe4cbde7465c222eaa84de",
    "calibration_legacy": "81700401e2fa6d370a86b6c1f5a48802bc5f1fc7",
    "calibration_d1_merge": "253de500a9ecc9827adddb1b0f20dea249efe7f1",
}
ROW_POLICY = {
    "released_pair": ("calibration_legacy", OLD, "console_released", "registered_legacy"),
    "old_consumer_new_source": ("calibration_d1_tested", OLD, "console_released", "offline_test_only"),
    "hardened_consumer_legacy": ("calibration_legacy", OLD, "console_candidate", "offline_test_only"),
    "hardened_consumer_d1": ("calibration_d1_tested", OLD, "console_candidate", "offline_test_only"),
    "new_package_legacy_wrapper": (None, NEW, "console_candidate", "incompatible_version_discriminator"),
    "new_package_new_consumer": (None, NEW, None, "planned_untested"),
}


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def validator(schema):
    Draft202012Validator.check_schema(schema)
    return Draft202012Validator(schema, format_checker=FormatChecker())


def reject_error(registry, action, label):
    try:
        action()
    except registry.RegistryError:
        return
    raise AssertionError(f"Registry accepted {label}")


def check_pin(path, expected):
    got = subprocess.check_output(["git", "-C", str(path), "rev-parse", "HEAD"], text=True).strip()
    assert got == expected, (path, got, expected)
    assert not subprocess.check_output(
        ["git", "-C", str(path), "status", "--porcelain", "--untracked-files=no"], text=True
    ).strip(), f"modified tracked checkout: {path}"


def check_matrix():
    matrix = read_json(MATRIX_DIR / "cal-021-d2-matrix.v0.1.json")
    schema = read_json(MATRIX_DIR / "cal-021-d2-matrix.v0.1.schema.json")
    v = validator(schema)
    v.validate(matrix)
    for name, pin in EXPECTED_PINS.items():
        assert matrix["revisions"][name] == pin, name
    assert matrix["revisions"]["calibration_d1_tested"] == "f609f827322a3142ba75e8a36a5279bb87fc606e"
    assert matrix["revisions"]["core"] == "afe1a831b950a4e6560f0285a0103fc4eaa343b7"
    assert matrix["revisions"]["ri_candidate"] == "563d923cd65805a2fc40b431635241871720932f"
    rows = {}
    for row in matrix["compatibility"]:
        assert row["id"] not in rows, "duplicate matrix row"
        rows[row["id"]] = tuple(row[k] for k in ("cal_source", "package_version", "consumer", "status"))
        if row["package_version"] == NEW:
            assert row["evidence"] is None
        else:
            assert row["evidence"] is not None
    assert rows == ROW_POLICY
    for flag in ("publication_authorized", "runtime_authority", "actuation_enabled"):
        altered = dict(matrix, **{flag: True})
        assert not v.is_valid(altered), flag
    for key, value in (("source_commit", "a" * 40), ("registry_descriptor", "modules/fake.json"),
                       ("publication_state", "released"), ("permissions_required", ["motor.write"])):
        altered = copy.deepcopy(matrix)
        altered["proposed_package"][key] = value
        assert not v.is_valid(altered), key
    print("PASS matrix: 6 exact compatibility rows, 7 negative publication/authority/schema checks")
    return matrix


def check_pinned_contracts(paths, registry, matrix):
    for name, path in paths.items():
        check_pin(path, EXPECTED_PINS[name])
    root = paths["registry"] / "registry"
    index = read_json(root / "index.json")
    descriptor_file = root / "modules" / MODULE / (OLD + ".json")
    descriptor = read_json(descriptor_file)
    validator(read_json(root / "schemas/index.v0.1.schema.json")).validate(index)
    validator(read_json(root / "schemas/module.v0.1.schema.json")).validate(descriptor)
    assert registry.catalog() == [descriptor]
    assert registry.entry(MODULE, OLD) == descriptor
    assert registry.entry(MODULE) == descriptor  # unique legacy baseline only
    assert descriptor["source"]["commit_sha"] == EXPECTED_PINS["calibration_legacy"]
    assert descriptor["version"] == matrix["registered_version"]
    assert descriptor["permissions_required"] == []
    assert descriptor["safety"] == {
        "non_actuating": True, "automatic_motion_on_load": False,
        "active_calibration_supported": False,
    }
    assert descriptor["integration"]["read_only_command"] == "inspect"
    assert descriptor["distribution"]["hosts"] == ["linux-x86_64"]
    reject_error(registry, lambda: registry.entry(MODULE, NEW), "unpublished 0.2.1")
    # Both historical sources still declare 0.2.0. This checks manifest
    # compatibility, NOT registration of D.1 at the old immutable version.
    for name in ("calibration_legacy", "calibration_d1_merge"):
        cal = paths[name]
        registry._verify_upstream_manifest(cal, descriptor)
        cargo = tomllib.loads((cal / "Cargo.toml").read_text(encoding="utf-8"))
        lock = tomllib.loads((cal / "Cargo.lock").read_text(encoding="utf-8"))
        manifest = read_json(cal / "module.openrgd.json")
        assert cargo["package"]["name"] == descriptor["distribution"]["package"]
        assert cargo["package"]["version"] == manifest["module_version"] == OLD
        assert [p["version"] for p in lock["package"] if p["name"] == "rgd-calibration"] == [OLD]
        assert set(manifest["capabilities"]) == set(descriptor["capabilities"])
        assert cargo.get("features", {}).get("default", []) == []
        report = read_json(cal / "contracts/passive-report.v0.2.schema.json")
        profile = read_json(cal / "contracts/calibration-profile.v0.1.schema.json")
        assert report["properties"]["report_version"]["const"] == matrix["contracts"]["passive_report"]
        assert profile["properties"]["schema_version"]["const"] == matrix["contracts"]["profile_schema"]

    d1 = paths["calibration_d1_merge"]
    for name in ("console_released", "console_candidate"):
        console = paths[name]
        wrapper = read_json(console / "contracts/console/v1/calibration-inspection.v0.4b.schema.json")
        assert wrapper["properties"]["schema_version"]["const"] == matrix["contracts"]["console_legacy_wrapper"]
        assert wrapper["properties"]["source_integrity"]["const"] == "canonical_not_verified"
        version = wrapper["properties"]["inspection"]["properties"]["module_version"]
        version_validator = validator(version)
        assert version_validator.is_valid(OLD)
        assert not version_validator.is_valid(NEW)
        agent = (console / "services/module-agent/calibration_passive.py").read_text(encoding="utf-8")
        assert re.search(r"(?m)^MODULE_VERSION\s*=\s*['\"]0\.2\.0['\"]$", agent)
        api = (console / "services/api/src/module_manager.rs").read_text(encoding="utf-8")
        assert re.search(r'body\.get\("module_version"\).*Some\("0\.2\.0"\)', api)
    for filename in ("passive-report.v0.2.schema.json", "calibration-profile.v0.1.schema.json"):
        assert (d1 / "contracts" / filename).read_bytes() == (
            paths["console_candidate"] / "services/module-agent/calibration-contracts" / filename
        ).read_bytes()
    print("PASS pinned contracts: registered legacy descriptor, both historical CAL manifests/locks, two Console wrapper/API/Agent version discriminators, candidate owner schema byte equality")
    return descriptor


def fixture_source(item, dest):
    """Synthetic package layout, deliberately NOT a real CAL source checkout."""
    dest.mkdir(parents=True)
    write_json(dest / "module.openrgd.json", {
        "module_id": item["id"], "module_version": item["version"],
        "non_actuating": True, "automatic_motion_on_load": False,
        "active_calibration_supported": False, "registry_status": "proposal-only",
        "capabilities": item["capabilities"],
    })
    (dest / "Cargo.toml").write_text(
        '[package]\nname = "rgd-calibration"\nversion = "' + item["version"] + '"\n',
        encoding="utf-8",
    )
    (dest / "Cargo.lock").write_text(
        '[[package]]\nname = "rgd-calibration"\nversion = "' + item["version"] + '"\n',
        encoding="utf-8",
    )
    (dest / "source-marker.txt").write_text("SYNTHETIC ONLY " + item["version"], encoding="utf-8")


def fixture_build(argv, *, cwd):
    assert argv == ["cargo", "build", "--release", "--locked", "--bin", "rgd-calibration"]
    target = cwd / "target/release/rgd-calibration"
    target.parent.mkdir(parents=True)
    # Never executed. Registry hashes the deliberately non-program bytes.
    target.write_bytes(b"SYNTHETIC NOT A PROGRAM\n" + (cwd / "source-marker.txt").read_bytes())
    return ""


def check_private_catalog(paths, registry, descriptor):
    # No real commit is assigned to 0.2.1. This synthetic descriptor is private
    # and is deleted with the temporary directory; its SHA is only a test token.
    with tempfile.TemporaryDirectory(prefix="cal-d2-private-") as tmp:
        temp = Path(tmp)
        original = paths["registry"] / "registry"
        private = temp / "registry"
        shutil.copytree(original, private)
        relative_old = "modules/" + MODULE + "/" + OLD + ".json"
        relative_new = "modules/" + MODULE + "/" + NEW + ".json"
        old_bytes = (private / relative_old).read_bytes()
        future = copy.deepcopy(descriptor)
        future["version"] = NEW
        future["source"]["commit_sha"] = "a" * 40  # unmistakably synthetic
        future["summary"] = "SYNTHETIC D.2 fixture, not a release"
        validator(read_json(private / "schemas/module.v0.1.schema.json")).validate(future)
        write_json(private / relative_new, future)
        index = read_json(private / "index.json")
        index["modules"].append({"id": MODULE, "version": NEW, "path": relative_new})
        validator(read_json(private / "schemas/index.v0.1.schema.json")).validate(index)
        write_json(private / "index.json", index)
        with mock.patch.object(registry, "REGISTRY_ROOT", private), \
             mock.patch.object(registry, "_fetch_pinned", side_effect=fixture_source) as fetch, \
             mock.patch.object(registry, "_run", side_effect=fixture_build) as build, \
             mock.patch.object(registry, "_host", return_value="linux-x86_64"):
            assert [x["version"] for x in registry.catalog()] == [OLD, NEW]
            assert registry.entry(MODULE, OLD) == descriptor
            assert registry.entry(MODULE, NEW) == future
            for fn in (registry.entry, registry.status_of, registry.resolve_binary, registry.install):
                reject_error(registry, lambda fn=fn: fn(MODULE), f"ambiguous selection in {fn.__name__}")
            fetch.assert_not_called()
            build.assert_not_called()
            prefix = temp / "prefix"
            assert registry.status_of(MODULE, version=NEW, prefix=prefix)["status"] == "not_installed"
            old = registry.install(MODULE, version=OLD, prefix=prefix, build=True)
            old_path = Path(old["binary"])
            old_receipt = old_path.parent.parent / "installed.json"
            old_content = old_path.read_bytes()
            old_meta = old_receipt.read_bytes()
            new = registry.install(MODULE, version=NEW, prefix=prefix, build=True)
            new_path = Path(new["binary"])
            assert old["status"] == new["status"] == "ready"
            assert old["version"] == OLD and new["version"] == NEW
            assert old["source_commit"] != new["source_commit"]
            assert old_path == prefix / "modules" / MODULE / OLD / "bin/rgd-calibration"
            assert new_path == prefix / "modules" / MODULE / NEW / "bin/rgd-calibration"
            assert old_path != new_path and old["binary_sha256"] != new["binary_sha256"]
            assert fetch.call_count == build.call_count == 2
            assert old_path.read_bytes() == old_content and old_receipt.read_bytes() == old_meta

            for ver in (OLD, NEW):
                reject_error(registry, lambda ver=ver: registry.install(
                    MODULE, version=ver, prefix=prefix, build=True), f"overwrite {ver}")
            assert fetch.call_count == build.call_count == 2
            assert registry.resolve_binary(MODULE, version=NEW, prefix=prefix)["binary"] == str(new_path)
            new_path.write_bytes(b"SYNTHETIC TAMPER")
            assert registry.status_of(MODULE, version=NEW, prefix=prefix)["status"] == "tampered"
            reject_error(registry, lambda: registry.resolve_binary(
                MODULE, version=NEW, prefix=prefix), "tampered new version, no fallback")
            rollback = registry.resolve_binary(MODULE, version=OLD, prefix=prefix)
            assert rollback["binary"] == str(old_path)
            assert rollback["binary_sha256"] == old["binary_sha256"]
            assert old_path.read_bytes() == old_content and old_receipt.read_bytes() == old_meta
            assert fetch.call_count == build.call_count == 2

            other = temp / "source-only"
            assert registry.install(MODULE, version=NEW, prefix=other, build=False)["status"] == "source_only"
            reject_error(registry, lambda: registry.resolve_binary(
                MODULE, version=NEW, prefix=other), "source-only resolution")
            assert build.call_count == 2

            # Version and source-commit disagreement are not healed or resealed.
            receipt = new_path.parent.parent / "installed.json"
            write_json(receipt, dict(read_json(receipt), version=OLD))
            assert registry.status_of(MODULE, version=NEW, prefix=prefix)["status"] == "tampered"
            for filename in ("module.openrgd.json", "Cargo.toml", "Cargo.lock"):
                def mismatched(item, dest, filename=filename):
                    fixture_source(item, dest)
                    target = dest / filename
                    target.write_text(target.read_text().replace(NEW, OLD), encoding="utf-8")
                broken = temp / ("bad-" + filename)
                with mock.patch.object(registry, "_fetch_pinned", side_effect=mismatched):
                    reject_error(registry, lambda: registry.install(
                        MODULE, version=NEW, prefix=broken, build=True), filename + " version mismatch")
                assert not (broken / "modules" / MODULE / NEW).exists()
            assert build.call_count == 2
            assert registry.resolve_binary(MODULE, version=OLD, prefix=prefix)["binary"] == str(old_path)
            assert old_path.read_bytes() == old_content and old_receipt.read_bytes() == old_meta
        assert (private / relative_old).read_bytes() == old_bytes
        assert (original / relative_old).read_bytes() == old_bytes
        assert not (original / relative_new).exists()
    print("PASS synthetic two-version Registry: explicit selection, isolated staged installs (mock fetch/build), no overwrite/fallback, tamper and source-only rejection, explicit old-version resolution, 3 package-version mismatch cases, unchanged historical bytes")


def main():
    parser = argparse.ArgumentParser()
    for name in EXPECTED_PINS:
        parser.add_argument("--" + name.replace("_", "-"), dest=name, type=Path, required=True)
    args = parser.parse_args()
    paths = {k: getattr(args, k).resolve(strict=True) for k in EXPECTED_PINS}
    sys.path.insert(0, str(paths["registry"]))
    from rgd_registry import core as registry

    matrix = check_matrix()
    descriptor = check_pinned_contracts(paths, registry, matrix)
    check_private_catalog(paths, registry, descriptor)
    for name, path in paths.items():
        check_pin(path, EXPECTED_PINS[name])
    print("PASS D.2 engineering strategy only; actual Registry distribution still 0.2.0. No real install, release, API/UI, hardware or owner promotion.")


if __name__ == "__main__":
    main()
