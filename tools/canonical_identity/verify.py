#!/usr/bin/env python3
"""CAL-021-B: optional, local Core-owned source integrity assessment.

Not a second canonical hash implementation, a hardware binding, a profile
validator, or a production Console endpoint. No writes to supplied inputs.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile

CORE_REF = "afe1a831b950a4e6560f0285a0103fc4eaa343b7"
PROFILE = "OPENRGD_SOURCE_TREE_SHA256_V1"
SCHEMA = "cal.canonical-identity.v0.1-candidate"
# Exact Git blob identities of the Core implementation and its local imports.
# These are source version checks, not signatures or a security sandbox.
CORE_BLOBS = {
    "canonical.py": "543f4da4754aa74a952aadd0c3a04724d77d0eab",
    "utils.py": "ed9ed2279b8a75362096a311e608b7e07bdef73d",
    "config.py": "9896ebe283844e4390fb163be52f63e0ae3dc11a",
    "visuals.py": "453098cb538bf6c2b19d7fde92cb2b16ee9ea6e0",
}
PACKAGE = "_cal021_pinned_core"
ROLES = ("description", "calibration", "actuation_topology", "hal_mapping", "sensor_fidelity")
MAX_ENTRIES = 256
MAX_TOTAL = 16 * 1024 * 1024
MAX_FILE = 4 * 1024 * 1024
MAX_REPORT = 4 * 1024 * 1024
DIGEST = re.compile(r"sha256:[0-9a-f]{64}\Z")


class BoundaryError(ValueError):
    def __init__(self, code):
        super().__init__(code)
        self.code = code


def read_regular(path, limit):
    # No final-component symlinks, devices, or unbounded reads. This wrapper
    # requires a stable operator-owned input, not a concurrently hostile tree.
    if not stat.S_ISREG(path.lstat().st_mode):
        raise BoundaryError("unsafe_source_entry")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0)
    with os.fdopen(os.open(path, flags), "rb") as source:
        meta = os.fstat(source.fileno())
        if not stat.S_ISREG(meta.st_mode):
            raise BoundaryError("unsafe_source_entry")
        if meta.st_size > limit:
            raise BoundaryError("source_size_limit")
        raw = source.read(limit + 1)
    if len(raw) > limit:
        raise BoundaryError("source_size_limit")
    return raw


def sha256(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def git_blob_id(raw):
    # Pin the version of upstream code only; NOT the RGD canonical hash.
    return hashlib.sha1(b"blob " + str(len(raw)).encode("ascii") + b"\0" + raw).hexdigest()


def load_pinned_core(checkout, private_dir):
    """Load the exact reviewed Core algorithm and its three local imports.

    Do not execute source from the input bundle or import arbitrary Core
    checkout modules. Trusted Python environment/dependencies are required.
    """
    upstream = checkout / "src/openrgd/core"
    files = {}
    for name, expected in CORE_BLOBS.items():
        raw = read_regular(upstream / name, MAX_FILE)
        if git_blob_id(raw) != expected:
            raise BoundaryError("core_code_pin_mismatch")
        files[name] = raw
    package_dir = private_dir / PACKAGE
    package_dir.mkdir()
    init = package_dir / "__init__.py"
    init.write_text("", encoding="utf-8")
    for name, raw in files.items():
        (package_dir / name).write_bytes(raw)
    if PACKAGE in sys.modules:
        raise BoundaryError("core_module_already_loaded")
    spec = importlib.util.spec_from_file_location(
        PACKAGE, init, submodule_search_locations=[str(package_dir)]
    )
    package = importlib.util.module_from_spec(spec)
    sys.modules[PACKAGE] = package
    spec.loader.exec_module(package)
    core = importlib.import_module(PACKAGE + ".canonical")
    if core.INTEGRITY_PROFILE != PROFILE:
        raise BoundaryError("core_profile_mismatch")
    return core


def snapshot_spec(bundle, target):
    if not bundle.exists() and not bundle.is_symlink():
        raise BoundaryError("source_tree_unavailable")
    if not stat.S_ISDIR(bundle.lstat().st_mode):
        raise BoundaryError("unsafe_source_entry")
    source = bundle if bundle.name == "spec" else bundle / "spec"
    if not source.exists() and not source.is_symlink():
        raise BoundaryError("source_tree_unavailable")
    if not stat.S_ISDIR(source.lstat().st_mode):
        raise BoundaryError("unsafe_source_entry")
    destination = target / "spec"
    destination.mkdir(parents=True)
    count = total = 0
    def fail_walk(error):
        raise error

    for directory, dirs, files in os.walk(source, followlinks=False, onerror=fail_walk):
        parent = Path(directory)
        relative = parent.relative_to(source)
        for name in sorted(dirs + files):
            count += 1
            if count > MAX_ENTRIES:
                raise BoundaryError("source_entry_limit")
            path = parent / name
            out = destination / relative / name
            mode = path.lstat().st_mode
            if stat.S_ISDIR(mode):
                out.mkdir()
            elif stat.S_ISREG(mode):
                raw = read_regular(path, min(MAX_FILE, MAX_TOTAL - total))
                total += len(raw)
                if total > MAX_TOTAL:
                    raise BoundaryError("source_size_limit")
                out.write_bytes(raw)
            else:
                raise BoundaryError("unsafe_source_entry")
    return destination


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise BoundaryError("manifest_duplicate_key")
        result[key] = value
    return result


def finite_float(raw):
    value = float(raw)
    if not math.isfinite(value):
        raise BoundaryError("manifest_nonfinite_number")
    return value


def no_constant(_):
    raise BoundaryError("manifest_nonfinite_number")


def preflight_manifest(core, spec):
    raw = read_regular(spec / "manifest.jsonc", MAX_FILE).decode("utf-8")
    # The pinned Core normalizes the first matching self-hash field.
    # Refuse ambiguous literal occurrences even in comments, instead of
    # changing Core semantics or silently accepting a different preimage.
    if raw.count('"integrity_hash_str"') != 1:
        raise BoundaryError("manifest_self_hash_ambiguous")
    value = json.loads(
        core.strip_jsonc(raw),
        object_pairs_hook=unique_object,
        parse_float=finite_float,
        parse_constant=no_constant,
    )
    meta = value.get("meta_group") if isinstance(value, dict) else None
    if not isinstance(meta, dict):
        raise BoundaryError("manifest_meta_missing")
    declared = meta.get("integrity_hash_str")
    if not isinstance(declared, str) or not DIGEST.fullmatch(declared):
        raise BoundaryError("manifest_digest_invalid")
    profile = meta.get("integrity_profile_str")
    if not isinstance(profile, str):
        raise BoundaryError("manifest_profile_missing")
    return profile, declared


def empty_assessment():
    return {
        "schema_version": SCHEMA,
        "integrity_profile": PROFILE,
        "core_reference_commit": CORE_REF,
        "core_code_pin_checked": False,
        "integrity_status": "not_checked",
        "declared_source_tree_sha256": None,
        "computed_source_tree_sha256": None,
        "indexed_file_count": None,
        "report_binding_status": "not_checked",
        "calibration_binary_sha256": None,
        "calibration_report_stdout_sha256": None,
        "calibration_report": None,
        "reasons": [],
        "bundle_completeness": "not_assessed",
        "canonical_schema_validated": False,
        "physical_instance_id": None,
        "robot_binding_verified": False,
        "profile_model_hash_bound": False,
        "physical_calibration_certified": False,
        "actuation_enabled": False,
    }


def assess_tree(core, spec, result):
    if not (spec / "manifest.jsonc").is_file():
        result["integrity_status"] = "manifest_missing"
        result["reasons"].append("manifest_missing")
        return None
    try:
        profile, declared = preflight_manifest(core, spec)
        result["declared_source_tree_sha256"] = declared
        if profile != PROFILE:
            result["integrity_status"] = "unsupported_profile"
            result["reasons"].append("unsupported_profile")
            return None
        # Single authority: do not reproduce discovery, normalization, source
        # index serialization, or root digest construction in Calibration.
        verified = core.compute_integrity(spec)
        if verified.profile != profile or verified.declared != declared:
            raise BoundaryError("core_manifest_interpretation_mismatch")
        if not DIGEST.fullmatch(verified.computed):
            raise BoundaryError("core_result_invalid")
        result["computed_source_tree_sha256"] = verified.computed
        result["indexed_file_count"] = verified.files_count
        result["integrity_status"] = (
            "source_tree_hash_match" if verified.matches else "source_tree_hash_mismatch"
        )
        if not verified.matches:
            result["reasons"].append("source_tree_hash_mismatch")
        return verified.index
    except BoundaryError as exc:
        result["integrity_status"] = "verification_failed"
        result["reasons"].append(exc.code)
    except (ValueError, OSError, TypeError, AttributeError, RecursionError):
        result["integrity_status"] = "verification_failed"
        result["reasons"].append("core_integrity_or_manifest_error")
    return None


def file_digest(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return "sha256:" + digest.hexdigest()


def expected_cal_paths(spec):
    paths = {}
    for role in ROLES:
        candidates = [
            spec / "01_foundation" / (role + extension)
            for extension in (".jsonc", ".json")
        ]
        present = [path for path in candidates if path.is_file()]
        if len(present) > 1:
            raise BoundaryError("ambiguous_cal_source")
        if present:
            paths[role] = "spec/" + present[0].relative_to(spec).as_posix()
    if "description" not in paths:
        raise BoundaryError("cal_description_missing")
    return paths


def match_report_to_index(report, spec, index):
    # Bind all actually inspected documents, not only description.jsonc.
    # The report was produced in this process's private snapshot. No external
    # report, digest claim or caller-supplied identity can enter this check.
    if not isinstance(report, dict) or report.get("report_version") != "0.2.0":
        raise BoundaryError("cal_report_version_invalid")
    if (
        report.get("profile_binding") != "no_profile"
        or report.get("profile_id") is not None
        or report.get("export_preview") is not None
        or any(report.get(key) is not False for key in (
            "current_robot_position_observed", "physical_drift_evaluated",
            "claims_physical_calibration_certified",
        ))
    ):
        raise BoundaryError("cal_report_authority_invalid")
    expected = expected_cal_paths(spec)
    entries = report.get("source_digests")
    if not isinstance(entries, list) or len(entries) != len(expected):
        raise BoundaryError("cal_report_source_set_mismatch")
    indexed = {} if index is None else {
        entry["path"]: entry["sha256"] for entry in index["files"]
    }
    all_indexed = index is not None
    seen = set()
    for entry in entries:
        if not isinstance(entry, dict):
            raise BoundaryError("cal_report_source_set_mismatch")
        role, path = entry.get("role"), entry.get("path")
        if role not in expected or role in seen or path != expected[role]:
            raise BoundaryError("cal_report_source_set_mismatch")
        seen.add(role)
        relative = path.removeprefix("spec/")
        raw_digest = sha256(read_regular(spec / relative, MAX_FILE))
        if entry.get("raw_file_sha256") != raw_digest:
            raise BoundaryError("cal_report_source_digest_mismatch")
        if indexed.get(relative) != raw_digest.removeprefix("sha256:"):
            all_indexed = False
    return "source_index_match" if all_indexed else "source_index_not_bound"


def inspect_snapshot(binary, spec, index, result):
    try:
        if not stat.S_ISREG(binary.lstat().st_mode):
            raise BoundaryError("cal_binary_invalid")
        result["calibration_binary_sha256"] = file_digest(binary)
        run = subprocess.run(
            [str(binary), "inspect", "--bundle", str(spec.parent)],
            cwd=spec.parent,
            env={"PATH": os.defpath, "LANG": "C.UTF-8"},
            capture_output=True,
            check=False,
            timeout=35,
        )
        if run.returncode != 0 or not 0 < len(run.stdout) <= MAX_REPORT:
            raise BoundaryError("cal_inspection_failed")
        report = json.loads(run.stdout)
        result["report_binding_status"] = match_report_to_index(report, spec, index)
        result["calibration_report_stdout_sha256"] = sha256(run.stdout)
        result["calibration_report"] = report
        if result["report_binding_status"] != "source_index_match":
            result["reasons"].append("cal_sources_not_fully_bound_to_core_index")
    except BoundaryError as exc:
        result["report_binding_status"] = "inspection_failed"
        result["reasons"].append(exc.code)
    except (ValueError, OSError, TypeError, KeyError, RecursionError, subprocess.TimeoutExpired):
        result["report_binding_status"] = "inspection_failed"
        result["reasons"].append("cal_inspection_or_report_error")


def assess(bundle, checkout, binary):
    result = empty_assessment()
    with tempfile.TemporaryDirectory(prefix="cal021-b-") as tmp:
        private = Path(tmp)
        try:
            spec = snapshot_spec(bundle, private / "bundle")
        except BoundaryError as exc:
            result["integrity_status"] = (
                "source_tree_unavailable" if exc.code == "source_tree_unavailable"
                else "verification_failed"
            )
            result["reasons"].append(exc.code)
            return result
        except OSError:
            result["integrity_status"] = "verification_failed"
            result["reasons"].append("source_snapshot_error")
            return result
        try:
            core = load_pinned_core(checkout, private)
            result["core_code_pin_checked"] = True
            index = assess_tree(core, spec, result)
        except (BoundaryError, OSError, ImportError, AttributeError):
            result["integrity_status"] = "verifier_unavailable"
            result["reasons"].append("trusted_core_unavailable_or_pin_mismatch")
            index = None
        inspect_snapshot(binary, spec, index, result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, required=True)
    parser.add_argument("--core-checkout", type=Path, required=True)
    parser.add_argument("--cal-binary", type=Path, required=True)
    args = parser.parse_args()
    report = assess(
        args.bundle.absolute(),
        args.core_checkout.absolute(),
        args.cal_binary.absolute(),
    )
    print(json.dumps(report, ensure_ascii=False, allow_nan=False, indent=2))
    return 0 if (
        report["integrity_status"] == "source_tree_hash_match"
        and report["report_binding_status"] == "source_index_match"
    ) else 2


if __name__ == "__main__":
    raise SystemExit(main())
