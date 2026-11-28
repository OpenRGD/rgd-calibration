# Calibration CLI contract — package 0.2.1 candidate

## Independent versions

- Package and executable metadata: `0.2.1`.
- Passive JSON output `report_version`: `0.2.0`.
- Historical profile `schema_version`: `0.1.0`.
- Registry descriptor contract: `0.1.0-candidate`; the actual registered version remains 0.2.0.

## Metadata only

```sh
rgd-calibration --version
# rgd-calibration 0.2.1
```

The single flag with no other arguments returns one plain-text line and exits 0. It is not an install, probe, motion or calibration command, and is not a new registry capability. The legacy registered 0.2.0 binary has no such option; do not require it for legacy inspection. This self-reported version is diagnostic metadata, not executable authentication; the consumer must also resolve an exact allowed descriptor, verify source pin and binary digest. Do not use the version string alone to authorize execution.

## Operational command

```sh
rgd-calibration inspect --bundle ROOT
rgd-calibration inspect --bundle ROOT --profile FILE --instance ID --model-hash sha256:HEX64
rgd-calibration inspect --bundle ROOT --out NEW_FILE
```

Exactly one `inspect` verb and one required `--bundle` option. Optional flags may each appear once; duplicates, unknown verbs/options, missing option values and option values beginning with "--" are rejected. ROOT is an explicit local directory, not a URL, ZIP or executable. The CLI uses the bounded CAL JSON/JSONC reader; it never requests a peripheral or runs an instruction from the model. Without `--out`, one JSON report plus newline goes to stdout. With `--out`, the CLI creates a new file only (never replaces an existing file), and does not print the report to stdout. Failures return 2 and diagnostic stderr, not a successful stdout report. An interrupted or failed write may leave a partial newly created file; it is not a committed profile. Do not forward arbitrary stderr remotely.

`--profile` reads an existing sealed profile, checks structural consistency/integrity and never rewrites it. Explicit `--instance` and `--model-hash` are caller assertions, not verified hardware identities or Core provenance. Only an appropriate sealed profile with matching claims can produce an unapplied delta preview. The Console no-profile route must use exactly `[resolved_binary, "inspect", "--bundle", private_snapshot]`, with no optional flags. Neither report nor preview authorizes motion or metrology. See [report schema](../contracts/passive-report.v0.2.schema.json) and [profile schema](../contracts/calibration-profile.v0.1.schema.json); validate cross references locally, not through HTTP.

## Capabilities are not executable commands

The existing seven manifest capabilities remain unchanged. Profile validation, sealing, persistence and binding inspection are library operations; passive inspection, coverage and read-only delta preview are report functionality. None creates extra CLI verbs. The optional RI candidate feature (default off), OCCHI laboratory tool, local Python Core verifier and test-only exchange example are not Registry executable commands, default runtime dependencies or granted device permissions. Build only the default passive executable: `cargo build --release --locked --bin rgd-calibration`. No install/activation on loading a model.

## Compatibility and migration

Package version changes do not imply report-schema or historical record acceptance compatibility. Tightened CAL-021-A/C parsing and evidence checks intentionally refuse some inputs accepted before. Preserve original records and source ZIP, never auto-reseal or normalize them. See [D.2 strategy](CAL_021_D2_COMPATIBILITY_VERSIONING.md). Console v0.4b requires module_version 0.2.0; it must not route 0.2.1 through that discriminator. A separate explicit-version consumer remains D.4.
