# CAL-021-B — Canonical Identity & Provenance (candidate)

## Purpose and scope

The Core owns the algorithm `OPENRGD_SOURCE_TREE_SHA256_V1`. Calibration does not implement a second canonical hasher, modify the manifest or invent a canonical hash from the five Foundation files. This is an optional local verifier/sidecar, not a new production Console endpoint or a normative Core contract.

Pinned upstream reference: [OpenRGD/openrgd@afe1a831b950a4e6560f0285a0103fc4eaa343b7](https://github.com/OpenRGD/openrgd/tree/afe1a831b950a4e6560f0285a0103fc4eaa343b7), specifically [canonical.py](https://github.com/OpenRGD/openrgd/blob/afe1a831b950a4e6560f0285a0103fc4eaa343b7/src/openrgd/core/canonical.py) and its three local imports. The actual reviewed source bytes are checked against their pinned Git blob IDs and loaded into a private temporary package. No canonical algorithm is copied into CAL or reimplemented; a different Core revision requires a reviewed update to the pin. Blob equality is version consistency, not a signed release or proof of Python runtime trust.

## Operator-invoked workflow

Prerequisites: trusted Python 3.10+ environment with the pinned Core toolchain's dependencies (Typer and Rich), a locally available checkout of the reviewed Core and a trusted, locally built CAL executable. Setup is separate; the verifier does not download anything or run a command from a model. Use only stable, operator-controlled local directories and trusted software, not attacker-controlled directories being modified concurrently.

```bash
cargo build --locked --bin rgd-calibration
python3 -I -B tools/canonical_identity/verify.py \
  --bundle /path/to/rgd-project \
  --core-checkout /path/to/reviewed/openrgd \
  --cal-binary target/debug/rgd-calibration
```

`--bundle` may be a project containing `spec/` or the actual `spec/` directory. A mirror-only `standard/` tree cannot establish source identity. The wrapper copies the entire available spec tree into a private temporary directory, with entry/file/total-size limits (256 entries, 4 MiB/file, 16 MiB total), rejects symlinks and special files, and never rewrites the source. Directory checks assume a stable local input, not a hostile concurrent filesystem. The Core's own implementation selects/indexes the canonical files, excludes generated aggregates and normalizes the manifest self-hash. CAL runs its existing fixed `inspect --bundle` command on the same snapshot; no `--profile`, `--out`, caller-supplied digest, network fetch, motor or device arguments are passed.

Manifest preflight conservatively rejects ambiguous literal self-hash fields, duplicate members, unsupported profile and invalid hash syntax. It uses the reviewed Core's JSONC stripping and adds rejection checks; it does not change Core's canonical hashing algorithm. Core manifest parsing may be stricter than the CAL Foundation parser, including trailing commas. Verification fails closed instead of normalizing the uploaded source.

## Three different statements

| Statement | What it actually means |
| --- | --- |
| `source_tree_hash_match` | The reviewed Core computes the declared digest from the available spec snapshot. Does NOT establish completeness, canonical schema validity, authorship, or physical identity. |
| `source_index_match` | All files read by CAL are present in that Core source index, and their exact raw bytes agree with CAL's report and the index. Can coexist with a root hash mismatch: the inspected bytes then match the *computed*, not declared, tree. |
| `source_index_not_bound` | Missing Core index, or one or more CAL inputs are outside that index (e.g. a local .json optional role). Never equate this with canonical identity. |

The JSON sidecar `cal.canonical-identity.v0.1-candidate` contains the original parsed passive report (if successful), distinct declared/computed source-tree hashes, local binary and report-stdout hashes, reference commit, reasons and explicit unknown/false flags. The stdout hash covers the original CAL output bytes, not a reserialization of the nested JSON. There is no `model_sha256` assignment or promotion of an existing CAL profile binding: the v0.1 profile's externally supplied model digest semantics are intentionally unchanged until the contract review.

Exit 0 requires both root match and source-index match. All other completed assessments return 2 and explicit statuses. Missing manifest, unsupported profile, unavailable verifier, unsafe source tree, parser failure and report mismatch do not claim success. Even exit 0 leaves `bundle_completeness=not_assessed`, `canonical_schema_validated=false`, `robot_binding_verified=false`, `profile_model_hash_bound=false`, `physical_calibration_certified=false`, `actuation_enabled=false`. A self-consistent partial tree can have a matching hash. Independent Core schema/completeness checks are a separate gate.

## Compatibility / verification

- The CAL-0.2 successful passive report schema, Rust API, package version and normal CLI are untouched. The sidecar is separate and opt-in; no RI feature or OCCHI capture is activated.
- CI checks the actual pinned Core checkout, full 76-file reference and original manifest commitment; negative tests cover mutations outside Foundation, comment changes, absent manifests, mirror-only data, duplicate fields, unsupported profile, unindexed JSON, wrong Core source, symlinks, invalid CAL and report digest mismatch.
- The sidecar schema is validated against actual generated outputs; deeper passive/profile schema conformance belongs to CAL-021-C.
- No installed Registry version/pin changes. The released Console Module Agent extracts only five Foundation documents; this is not enough to compute the entire Core source hash. A future reviewed consumer must supply the full, bounded original spec snapshot (not reconstruct it from the Foundation or JSON5 projections), use a trusted Core runtime, and preserve original ZIP hash separately.
- A local binary digest is not a signature; the standalone verifier deliberately takes an explicit trusted CAL executable, rather than claiming installation verification. Registry-mediated resolution and consumer rollout remain CAL-021-D/E.

Rollback: do not invoke the optional wrapper. Existing CAL-0.2 CLI and Registry remain unaffected. No hardware, Canonical Core, PERIPH, RI or Console owner changes.
