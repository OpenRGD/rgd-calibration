# CAL-021-A — Bundle & Parser Compatibility

**Candidate patch; no release or deployment.** Reviewed source baselines:

- CAL main `dab59d6908cd932c23df13a9b9ec2d9263bec505`.
- [Console v0.4.3](https://github.com/OpenRGD/rgd-console/tree/a0ef857cf425c1bb14ad8f62acec21f8c1324866), Rust `services/api/src/model_import.rs` (JSON5), Python `services/module-agent/calibration_passive.py` (bounded Foundation extraction, fixed argv and report validation).
- [Registry v0.1](https://github.com/OpenRGD/rgd-modules/tree/0268b3748042d04fd3561553616388368881e04a), installed CAL `0.2.0` remains pinned to `81700401e2fa6d370a86b6c1f5a48802bc5f1fc7`.
- [Core integrity implementation](https://github.com/OpenRGD/openrgd/blob/afe1a831b950a4e6560f0285a0103fc4eaa343b7/src/openrgd/core/canonical.py), owner of the canonical source-tree hash. This PR does NOT calculate or replace that algorithm.

## Common input contract

| Aspect | CAL candidate | Released Console v0.4.3 |
| --- | --- | --- |
| Container | Explicit local directory; no ZIP extraction in CAL | .zip or .rgd containing ZIP; Agent extracts selected files |
| Source layout | spec/01_foundation first; standard/ mirror, direct 01_foundation, direct directory supported | exactly one spec/01_foundation/description.jsonc at archive root or one top-level wrapper |
| .json | Strict UTF-8 JSON object | not a source-profile archive input |
| .jsonc | Strict JSON plus comments and trailing comma after a real value | JSON5 importer is broader; Agent passes exact bytes to CAL |
| JSON5-only constructs | rejected, never silently rewritten | importer can accept unquoted keys, single quotes and other JSON5 forms |
| Duplicate decoded keys | rejected recursively, including escaped aliases | current importer does not enforce this common restriction |
| Missing optional role | report finding, no invented values or cross-root backfill | Agent extracts available allowlisted .jsonc only |
| Per-file limit | 4 MiB, read bounded | 2 MiB for extracted Foundation, ZIP 8 MiB compressed / 16 MiB uncompressed / 256 entries |
| Integrity | digest of exact raw file bytes | original ZIP SHA-256 and CAL binary hash; neither proves canonical integrity |

The CAL loader accepts a mixture of .json/.jsonc *different roles* in a single local directory (legacy fixtures), but rejects the same role in both extensions. A valid spec root takes precedence over the generated standard mirror. A malformed preferred description is an error, never a reason to select a different valid mirror. A stable, private snapshot is assumed; ordinary filesystem checks are not a sandbox against an attacker racing file replacement.

No support for unquoted keys, single quotes, hexadecimal numbers, NaN/Infinity, comment-token concatenation, comma-only objects/arrays or missing values. Reject invalid UTF-8, duplicated object keys and unsafe source paths. JSONC normalization changes only comment/trailing-comma bytes in memory; original files remain unchanged. Native serde_json numeric representation remains in use, not a new exact metrology numeric standard.

## Partial Foundation and errors

A passive inspection can succeed on a partial Foundation **without** a manifest, kernel or optional source documents if the description provides a nonblank `meta_group.model_id_str` and explicit object maps for joints and sensors (empty is permitted; missing is not equivalent to empty). Joint/sensor entries must be objects. Optional files get individual `optional_source_not_provided` findings. This is **not** canonical validation or proof that the robot has no joints/sensors. Never fabricate a model ID from the Console upload SHA or display name.

Structured error tokens in existing human-readable stderr (exit 2, no success report): `CAL_BUNDLE_SYNTAX`, `CAL_BUNDLE_REQUIRED_FIELD`, `CAL_BUNDLE_AMBIGUOUS_SOURCE`, `CAL_BUNDLE_PATH`, `CAL_BUNDLE_SIZE`. Do not expose arbitrary stderr/paths in a remote UI. The CAL-0.2 CLI argv and successful report schema/version `0.2.0` are unchanged. No active mode, profile commit, hardware observation, RI feature activation or depth authorization.

## Validation and reproducibility

- Rust `tests/bundle_compatibility.rs`: shared syntax corpus, raw-byte hashes, empty/partial Foundation, nested duplicate names, local source layout, no cross-root backfill, symlinks, bounds.
- `tests/fixtures/cal-021-a/bundle-syntax-corpus.v1.json` is a synthetic, portable expected acceptance corpus; independent Console importer compatibility can reuse it.
- `scripts/test_console_bundle_compat.py` exercises **actual** pinned Console Module Agent ZIP extractor and report validator against the built candidate CAL binary, with the actual pinned Registry imported but no install. GitHub Actions retrieves pinned public repos/build dependencies at setup; test execution is local. This does **not** execute the Rust Console importer, entire Console API/UI, Docker or a deployed host. The shared subset is a CAL policy; broader JSON5-only import acceptance remains possible in the current Console.
- Original default/RI-feature Rust suites and pinned Core reference regression remain required. No changes to upstream hardware owner code or Registry descriptor.

## Handoff to Console owner (CAL-021-C/D)

1. Keep original ZIP and raw source bytes; do not normalize JSON5 into a new bundle or silently recompute its canonical identity.
2. For existing imported models that are outside the common syntax subset, expose *Calibration incompatible* rather than invalidate the whole Console model. Display only allowlisted stable failure categories, not arbitrary child stderr. The released Module Agent still maps child failure to a generic error.
3. Make a separate reviewable check of the full importer/agent selection: same-root .json siblings and optional .json-only roles are currently outside the Console .jsonc allowlist; do not advertise CAL's direct-directory layouts as Console ZIP support. Reject ambiguous/unsupported Foundation source layout explicitly before extraction in the future consumer change.
4. Canonical source-tree verification requires full `spec/`, including non-Foundation sources, using the Core authority (CAL-021-B). Current selective extraction cannot establish this digest.
5. Keep `console.calibration-inspection.v0.4b` wrapper and `passive-report.v0.2` as-is; exercise new compatibility before any CAL version pin or Console deployment. No updates to PERIPH, RI, Somatic or device owners.

**Acceptance for A:** candidate CAL input rules and actual Agent boundary tests pass; wider cross-repo importer enforcement and physical host acceptance remain separate gates. Rollback: keep Registry on the unchanged CAL-0.2.0 pin.
