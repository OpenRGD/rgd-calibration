# CAL-021-D.1 — Cross-Repository Reconciliation

**Candidate engineering review — 2026-11-27.** All three CAL-021-C owner returns received and reconciled. This wave adds independent cross-repository regression tests and documents the decisions. It does not promote a candidate interface to a standard, change a runtime contract or distribute a new module.

## Immutable evidence and revisions

| Owner | Reviewed response / revision | Owner-side evidence |
| --- | --- | --- |
| Console | [return](https://github.com/OpenRGD/rgd-calibration/pull/9#issuecomment-7223254883), [Draft #23](https://github.com/OpenRGD/rgd-console/pull/23) at `168e99c2ba057ae78bfe4cbde7465c222eaa84de`, base `b331dd32415eeda0d107676920a6dea91170cc4f` | [pinned consumer](https://github.com/OpenRGD/rgd-console/actions/runs/46033283350), [Foundation](https://github.com/OpenRGD/rgd-console/actions/runs/46033283343) |
| Core | [return](https://github.com/OpenRGD/rgd-calibration/pull/9#issuecomment-7223114804), `afe1a831b950a4e6560f0285a0103fc4eaa343b7`, toolchain 0.1.1 | Static authority-boundary review; no new owner test execution or source changes claimed |
| Reality Ingress | [return](https://github.com/OpenRGD/rgd-calibration/pull/9#issuecomment-7191220536), [Draft RI-A02B #9](https://github.com/OpenRGD/rgd-reality-ingress/pull/9) at `563d923cd65805a2fc40b431635241871720932f` | [native compatibility CI](https://github.com/OpenRGD/rgd-reality-ingress/actions/runs/45813903783) |
| Calibration | reviewed C head `646dbc05bcefd15942b25bb55b3c99096f0cccc9`; consolidated main `6fb88d99e566ec28bb77c878f7d4077a5b78e820` | [historical CAL exchange](https://github.com/OpenRGD/rgd-calibration/actions/runs/45806326568); D.1 is a separate suite |
| Registry | `0268b3748042d04fd3561553616388368881e04a`; descriptor `org.openrgd.calibration@0.2.0` pins legacy CAL `81700401e2fa6d370a86b6c1f5a48802bc5f1fc7` | Actual descriptor read in tests, status and binary resolution injected; no installation |

Old CAL-021-A/B/C workflows and pins are preserved; the previous behavior of Console v0.4.3 remains documented. No automatic repin to an owner's moving branch.

## Reconciliation decisions

### Console — candidate remediation

The owner implementation adds local full CAL schema validation (identical owner schema bytes), explicit no-profile context, digest comparison against private extracted bytes, local JSONC admission and no implicit archive repair. Keep passive-report v0.2.0 and wrapper console.calibration-inspection.v0.4b unchanged in this wave.

D.1 builds both the released binary and consolidated candidate source; executes the **actual owner consumer test** unchanged (22 syntax fixtures; 19 JSONC route cases per executable, 3 accepted and 16 refused; five-file byte digest check); independently tests both real CLIs through the new Agent against the full Core reference ZIP. It exercises the three C-era validator gaps and further wrong-digest, missing/duplicate/foreign source, inconsistent coverage, history, physical-authority and private-snapshot tamper cases. A new owner rejection may occur before subprocess execution: this is expected, not a parser regression. The current route remains the intersection of legacy/new parser behavior, including refusal of CR-only line comments despite acceptance by the new CLI. This is a migration concern, not grounds to alter old inputs. Same-root unsupported JSON files must not be silently ignored.

This is **candidate compatibility**, not installed compatibility. Console #23 remains draft. Keep its original version identity in the test wrapper: injected candidate resolution is explicitly a test, not evidence that candidate source was published as the old version.

### Core — ownership boundary confirmed; evolution deferred

Retain the actual pinned Core-owned compute_integrity/index implementation and CAL conservative preflight. Never use --write for checking. No second canonical hasher; no inference of completeness, schema validity, signed release, hardware identity, live pose or physical calibration from a root match. Do not change legacy profile model_sha256 or reseal saved data automatically.

D.1 compares the candidate's 76-file Core assessment and actual five Foundation file digests through both Console candidate/CLI pairings. The ZIP digest and canonical root must remain distinct. Console's existing wrapper retains canonical_not_verified; the independent CAL sidecar remains opt-in. Full bounded original spec delivery, archive snapshot collision/race prevention and optional sidecar transport are separate work; do not reuse the Boot Manager boot operation as CAL verification.

CORE-CAL-01A (versioned read-only assessment/index API) is a future improvement, **not a blocker for packaging the current passive CLI**. Keep completeness/schema and physical binding unknown where unassessed; CORE-CAL-01B/C and normative safety/alias issues remain separate. No Core source edit or promise of acceptance in this PR.

### RI — native integer compatibility, not a new wire format

Accept RI-A02B's proposed native-u64 subset as a **candidate compatibility target** for the three counters, not as an accepted canonical contract. Exact nonnegative integer values from 0 through 2^64-1; reject bools, floats (including integral float representation), strings and overflow. Keep the historical RI v0.1 schema and its original interpretation. Above 2^53-1, JavaScript Number consumers must use a reviewed lossless representation or reject the unsupported transport; no rounding, clamping, silent decimal-string conversion or alteration of historical IDs. Unicode code-point lengths are not byte budgets.

Run RI's actual owner test against the consolidated CAL exporter (27 variants, 44 independently validated sidecars expected); rerun the C producer-to-CAL exchange on the candidate RI checkout. Verify the original consumer schema and producer blob identities. The native helper is a candidate test boundary only; no new intake, A02 session resolver, A04 authorization/reader, A05 verification, source attestation, Chronograf or History authority. CAL continues to own only profiles/revisions/sidecars. Never grant metric depth or media access from a ref/hash.

## Acceptance and test invocation

See `.github/workflows/cal-021-d1-reconciliation.yml` and `scripts/test_d1_reconciliation.py`. Checkout/build setup can use network; tests use local sources and synthetic records only. The new suite validates exact source checkouts, owner schema equality, frozen RI blobs, actual Core identity, both CAL CLI binaries, actual candidate Agent with injected Registry installed state/resolution, and actual RI producer and native compatibility tests. The Console API envelope is a schema fixture, not a live API response. No new Rust features, manifest version or runtime code.

D.1 implementation can be reviewed once the new workflow and unchanged A/B/C/Foundation suites pass on the exact PR head. Record actual counts and CI links in PR review; do not infer pass from this plan. Independent consumer checks are scoped to the tested candidate heads. Upstream PRs remain independently controlled.

## D.2–D.4 and release gates

| Gate | Disposition |
| --- | --- |
| Owner returns received | Reviewed (three of three); no presumed upstream merge |
| Console validator gaps | Candidate remediation; independent tests required, no production acceptance |
| Numeric domain | Candidate Python/Rust agreement; JS transport and future versioned policy still open |
| Canonical verifier | Existing pinned implementation retained; new Core API deferred; completeness unassessed |
| D.2 | Specify explicit version matrix, report ABI, old/new parser intersection and migration policy; no automatic pin change |
| D.3 | Prepare reviewed CAL-0.2.1 package/manifest with unchanged non-actuating interface; no premature publication |
| D.4 | Review parallel install and rollback plan with Console/Registry owners |
| E/F | Real host/UI/install/offline/restart/rollback evidence, packaging/provenance and independent release approval |

No automatic merge of external PRs, Registry update, module deployment, historical record mutation, hardware ownership change or physical calibration certification. The CAL package and installed Registry pin remain 0.2.0. No dependency on an unimplemented Core API to advance D.2 planning.
