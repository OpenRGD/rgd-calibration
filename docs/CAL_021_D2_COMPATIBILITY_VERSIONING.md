# CAL-021-D.2 — Compatibility Matrix & Versioning Strategy

**Engineering candidate, 28 November 2026.** D.1 is merged in CAL at `253de500a9ecc9827adddb1b0f20dea249efe7f1` ([PR #11](https://github.com/OpenRGD/rgd-calibration/pull/11), [10 green workflow runs](https://github.com/OpenRGD/rgd-calibration/pull/11)). This wave establishes release planning and offline checks, not a runtime selector, new Registry entry, package release, or acceptance of other owners' Draft PRs.

The machine-readable [matrix](compatibility/cal-021-d2-matrix.v0.1.json) is a historical engineering snapshot checked by its [local schema](compatibility/cal-021-d2-matrix.v0.1.schema.json). It is **not** a distribution catalog, signature, authorization service or second source of truth for the module registry. Registry owners keep publication authority. The prospective package has no source commit or descriptor assigned until D.3/D.4.

## Independent version axes

| Axis / owner | Registered state | Proposed 0.2.1 behavior |
| --- | --- | --- |
| CAL package + Cargo.lock + module manifest / CAL | 0.2.0 | All three must agree on 0.2.1 at D.3; version not changed here |
| Registry descriptor / Registry | schema 0.1.0-candidate, exact CAL 0.2.0 pin | Append a **new** 0.2.1 descriptor after immutable package commit exists; retain 0.2.0 byte-for-byte |
| Passive CLI/report / CAL | inspect; report 0.2.0 | Same report version and command shape; stricter input acceptance and repaired schema reference documented, no promise of identical acceptance or bytes |
| Persistent profile / CAL | schema 0.1.0 | Same serialized format; tightened reference checks can reject formerly accepted records |
| Console wrapper + Agent / Console | console.calibration-inspection.v0.4b / console.passive-calibration.v0.4b, module_version exactly 0.2.0 | Keep legacy route and discriminator; propose a **separate owner-reviewed versioned inspection contract** for explicit 0.2.1 selection |
| Canonical Core / Core | OPENRGD_SOURCE_TREE_SHA256_V1, toolchain 0.1.1 pinned | Retain independent optional canonical-identity assessment; no completeness/physical-binding claim |
| RI / RI + CAL | frozen RI v0.1 candidate + CAL RI candidate sidecar | Off by default; same native Python/Rust u64 candidate subset; transport above JS safe integer remains gated |

The expected package version is 0.2.1 because this is a pre-1.0 hardening release, not because all historical behavior is compatible. The stricter admission and profile reference rules are intentional compatibility changes. If D.3 identifies an actual report wire incompatibility, stop and review a new report version and consumers instead of silently changing v0.2.0. Similarly, never claim old records are valid under new checks merely because their schema number matches.

## Compatibility matrix

| Pair | Status | Interpretation |
| --- | --- | --- |
| Registered CAL 0.2.0 at `81700401...` + released Console v0.4.3 | Registered legacy baseline | Existing distribution and rollback target, not a new E host test |
| Consolidated A/B/C/D.1 source (still 0.2.0) + old Console Agent | Historical offline compatibility | A/C tests pass at pinned revision; old validation gaps remain |
| Registered 0.2.0 + Console #23 at `168e99c2...` | Offline candidate-compatible | D.1 actual Agent, injected Registry resolution, conservative parser intersection |
| Consolidated D.1 source + Console #23 | Offline candidate-compatible | Same test-only resolution and original report version; not an installed release |
| Future 0.2.1 + current v0.4b wrapper/API | Incompatible version discriminator | Do not run it while labeling it 0.2.0; installability alone is insufficient |
| Future 0.2.1 + explicitly version-aware Console | Planned, untested | D.3 package + D.4 owner integration; E/F independent acceptance |

See [D.1 actual candidate CI](https://github.com/OpenRGD/rgd-calibration/actions/runs/46048542205). The RI native helper was exercised, not the A02 session resolver or authorized media reader. The Core hash sidecar was tested separately; the Console's five-file extraction remains noncanonical. External candidate heads are fixed, not moving branch aliases.

## Explicit selection and compatibility of consumers

The current Registry implementation already accepts multiple entries for a module, sorts its catalog, and requires an explicit version when there is more than one. It installs under `<prefix>/modules/<id>/<version>/`. Do not infer a preferred release from lexical sort, highest semver, installed state, "ready", or last install. Keep the current descriptor and its commit immutable. The current index/descriptor schema only accepts numeric major.minor.patch: do not invent a prerelease descriptor such as 0.2.1-rc.1 under that schema; any RC is a review state, not a different registered version. Registry publication governance and signed provenance belong to the Registry/Core owners.

D.4 needs a reviewed Console/API/Agent contract carrying an explicit allowed module identity and version to the server-side resolver, returning the **actual** selected package version, source commit and executable digest from the checked resolution. No arbitrary executable/path/argv parameters. Proposed next versioned wrapper is separate from v0.4b; its final ID/shape is Console-owned. Retain old v0.4b endpoint semantics, including module_version 0.2.0 and canonical_not_verified. For a request to use 0.2.1 with an old/mixed-version Agent or API, fail closed with a stable unsupported-version diagnostic, not an automatic downgrade. Activation is operator-approved, not install-triggered. A failed 0.2.1 must not silently resolve 0.2.0; an explicit rollback operation may select 0.2.0 after a fresh integrity check.

**D.3 package requirements:** update Cargo.toml package version, root Cargo.lock entry and module.openrgd.json module_version together; verify manifest/capabilities against actual registry implementation; maintain default features empty, build with `--locked --release --bin rgd-calibration` and publish no test executable, RI/OCCHI entry point or Python verifier as a CLI capability. Add actual CLI argument/version documentation that distinguishes seven library/inspection capabilities from the one executable command. No new permissions; candidate status unchanged. The optional tools remain in source only unless separately reviewed. Create an immutable commit and exact digest evidence before proposing any new Registry descriptor. D.1 tests intentionally injected candidate resolution into the legacy wrapper; after D.3 keep that historical evidence, explicitly label such testing and do not assert it is actual 0.2.1 deployment.

## Parser, profile and cached-report migration

- Preserve the original ZIP, imported model identifiers and raw document hashes. The current Console route uses the legacy/new JSONC intersection; it refuses CR-only line comments although the new CLI accepts them. D.4 may allow the wider new syntax **only** after explicit selection of the new version and exact tests; no normalization/repackaging, fallback or invented aliases. Strict JSON directory support remains separate from .jsonc-only ZIP admission.
- On a copied workspace, inventory old profile IDs, seals and structural status without mutation. Verify old content using its matching historical contract/runtime where needed. Under new validation, duplicate evidence links or dangling zero-reference evidence are actionable incompatibilities, not proof of forgery. Keep failed historical records read-only, retain provenance and display review required. Never "repair" them by deleting references, changing the old seal, fabricating measurements or promoting physical validation.
- Any approved correction is a separate immutable new revision with predecessor linkage, evidence and renewed review under the appropriate policy. The existing revision helper verifies the parent, so do not route an old record rejected by the new validator into it blindly. A reviewed legacy-import/migration adapter may be necessary; keep that decision/gate open. No automatic migration in D.2/D.3.
- Old cached Console snapshots remain historical and read-only. Do not rewrite module_version, binary digest, source hash or certification fields. If a preview lacks current required context/digests, mark unusable for current inspection and require a new explicit inspection. New selection affects only future operations; neither old nor new snapshot becomes a physical observation.
- Core identity remains an optional separate sidecar. Full original bounded spec snapshot, current Core-owned read-only verifier, manifest/required-domain admission and independent completeness check stay separate from current report acceptance. CORE-CAL-01A is not a packaging dependency; do not claim full canonical validation from selective extraction.
- RI numeric compatibility uses exact native integers, not rounded JavaScript numbers. The current passive Console report does not activate RI or require new RI wire transport. A reviewed lossless/versioned codec or explicit rejection remains necessary for a future RI web consumer.

## D.2 verification scope

The new offline test loads the **actual pinned Registry** and Console schemas, validates the local matrix and checks historical descriptor/version/manifest/lock/contract pins. It characterizes the existing v0.4b version discriminator. In a disposable, explicitly synthetic two-version catalog, exercise actual registry selection/status/resolve/install-path behavior with fetch/build mocked: explicit versions, no fallback, isolated directories, refusal of overwrite, manifest/lock mismatch, tampered new binary, explicit resolution of old version and preservation of old bytes. The fake 0.2.1 descriptor and synthetic receipts are never published or used with actual module installation. These tests are not D.4 real install/rollback acceptance. No physical device, Console API server, Docker, live robot or motor command.

D.3: package preparation PR; D.4: coordinated version-aware consumer and parallel-install/migration/rollback implementation; E: supported-host installation, API/UI/mixed-image, offline restart and rollback with retained volumes; F: reviewed release decision and distribution provenance. A green D.2 does not authorize any of them automatically.

[Registry/Console handoff](handoffs/CAL-021-D2-TO-REGISTRY-CONSOLE.md) identifies what each owner must do before publication.
