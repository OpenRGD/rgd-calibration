# CAL-021-D.3 — Package Preparation

**Source package candidate, 28 November 2026.** Merged in [PR #13](https://github.com/OpenRGD/rgd-calibration/pull/13) at f912a961f55bb051022eadc25e1ebd8b95e8f4ef after 14 green workflows. The accepted, independently built head 292d42aab5e04392b736693233811734bffb603e remains the exact D.4 test pin; no new catalog entry. See [D.4 integration](CAL_021_D4_INTEGRATION_MIGRATION.md). D.2 is merged in [PR #12](https://github.com/OpenRGD/rgd-calibration/pull/12) at 68f5b855e10d4549f639c5044d99951f6a61b78e; D.3 prepares 0.2.1, without publishing it to the Registry, crates.io or a GitHub release.

## Package identity

| Component | Candidate |
| --- | --- |
| Module ID | org.openrgd.calibration |
| Cargo package, root lock entry, module manifest | 0.2.1 |
| Passive report ABI | 0.2.0 |
| Profile schema | 0.1.0 |
| Default features | none |
| Operational CLI | inspect |
| Metadata | --version, plain text, no bundle access |
| Registry state | proposal-only; installed/distributed version remains 0.2.0 |
| Build target for this wave | Linux x86_64, Rust 1.90.0 (matches Console Agent toolchain) |

No dependency resolution changes, new library capabilities, new authority or default RI/OCCHI/Core verifier activation. Existing default and all-feature tests remain required. See [CLI contract](CLI_CONTRACT.md) and [D.2 strategy](CAL_021_D2_COMPATIBILITY_VERSIONING.md). The report/profile/RI schema bytes and Rust library are preserved. Legacy profile validation changes from earlier waves remain documented; historical records are not automatically migrated or resealed.

## Actual package verification

The D.3 workflow checks out the exact candidate head (not a PR merge ref), builds the consolidated D.2 baseline and the candidate in release mode with locked dependencies and the default feature set. It invokes the actual pinned Registry's manifest verifier with an **in-memory provisional descriptor** derived from the existing descriptor. The object is never published; the existing old descriptor rejects the new source version. All three package version declarations, identical dependency lock entries, capabilities and non-actuating safety flags are checked.

Tests run the real release-mode executable: metadata version, passive-minimal and Core reference reports byte-identical to the pre-bump D.2 baseline, original source unchanged, schema validation, explicit output no-overwrite, malformed CLI/unknown verbs and the existing syntax corpus. This is version-bump regression compatibility, not a proof that every input accepted by the original registered 0.2.0 is accepted by the new code.

CI C and D.1 now explicitly build the **historical reviewed CAL commits** for their legacy wrapper experiments rather than mislabel the new package version as 0.2.0. The external pins and the reviewed test scripts are unchanged. A/B and Foundation continue testing current source. The D.2 matrix remains a dated planning snapshot; do not rewrite its historical evidence.

## Build evidence

A successful D.3 run retains a short-lived GitHub Actions review artifact: the real passive executable, build inputs (manifest, Cargo.toml/lock), passive/profile schemas, CLI documentation, license, BUILD-EVIDENCE.json and SHA256SUMS. Evidence records actual source commit, Git tree ID, toolchain, build command and file SHA-256 values, and explicitly denies publication, signature verification and actuation. This is **not** a signed, independently reproducible release, and the Registry continues distributing from its exact source-commit descriptor, not this CI artifact. The test-only example, RI feature, camera tool and Python verifier are not distributed as executables. Do not install on a robot from this artifact.

## Next gate

D.4 is separate: after accepting the reviewed package source, give Registry/Console owners its exact commit and evidence. Preserve the immutable 0.2.0 descriptor/pin. Console must select the exact version, verify the resolved source and binary, and return the actual package version using its own new versioned wrapper; v0.4b remains 0.2.0-only. No automatic downgrade. No source/Registry/Console rollout before review. E/F cover real supported-host installation, API/UI, mixed-version failures, offline restart, rollback, retention and production provenance.

No Core, RI, PERIPH, device or motion changes. No physical calibration is established.
