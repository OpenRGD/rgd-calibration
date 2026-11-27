# CAL-021 A/B/C — Consolidation and owner response register

**Review snapshot: 2026-11-27.** This is a planning and engineering evidence record, not a release certificate or a new runtime contract.

## Consolidated CAL source

| Wave | Reviewed PR | Merge commit | Historical CI evidence |
| --- | --- | --- | --- |
| A — Bundle & Parser Compatibility | [#7](https://github.com/OpenRGD/rgd-calibration/pull/7) | e66e4b8569457d89d7d4c5987d028bfdb13135bd | 45 default / 60 opt-in Rust tests; 22-case syntax corpus; 27 Agent boundary checks |
| B — Canonical Identity & Provenance | [#8](https://github.com/OpenRGD/rgd-calibration/pull/8) | 2e3a65d3883c9f3a24dbb4450b4fb9f0d34f385e | [15 Core-owned identity tests](https://github.com/OpenRGD/rgd-calibration/actions/runs/45796684407); full pinned 76-file Core reference |
| C — Contract Validation | [#9](https://github.com/OpenRGD/rgd-calibration/pull/9) | 8268a621dc2b14a22473830f19d448f32236d9b1 | [Eight green workflows at reviewed head 646dbc05bcefd15942b25bb55b3c99096f0cccc9](https://github.com/OpenRGD/rgd-calibration/pull/9); [actual contract exchange CI](https://github.com/OpenRGD/rgd-calibration/actions/runs/45806326568) |

CAL-021-C exercises actual RI producers, actual Core and actual Console Module Agent with injected Registry resolution. It validates seven owner schemas, twenty Rust association sidecars and fourteen rejected inputs. It characterizes three Console v0.4.3 validator gaps and one RI integer-domain mismatch. The API wrapper is synthetic; the RI-A02 resolver itself is not executed; no deployed Console, real Registry installation or physical measurements are established.

All three code waves are in main. The tested external revisions remain historical pins; do not silently change the baseline after an owner sends a new candidate.

## Owner response register

| Owner | Outbound | Return status at this review | Gate |
| --- | --- | --- | --- |
| Console | [CAL-021-C→Console](handoffs/CAL-021-C-TO-CONSOLE.md) | Sent via owner conversation; return not yet received in this CAL coordination record | Validate full CAL report and route context, compare extracted bytes/digests, JSONC/JSON5 incompatibility UX, full source snapshot boundary |
| Core | [CAL-021-C→Core](handoffs/CAL-021-C-TO-CORE.md) | Sent via owner conversation; return not yet received in this CAL coordination record | Supported Core verifier API/version, manifest parsing, completeness/schema vs root identity; preserve physical binding distinction |
| Reality Ingress | [CAL-021-C→RI](handoffs/CAL-021-C-TO-REALITY-INGRESS.md) | [Return received](https://github.com/OpenRGD/rgd-calibration/pull/9#issuecomment-7191220536); RI-A02B proposal reviewed in principle, candidate only | Joint numeric-profile/transport review and pinned consumer retest; reader authority remains RI-A04/A05 |
| Registry | [CAL-021-C→Registry](handoffs/CAL-021-C-TO-REGISTRY.md) | Deferred until D | Versioned descriptor, explicit version selection, compatibility and rollback; preserve old pin |

The absence of a return in this register is not a claim that no work has been done in another conversation or repository.

### RI-A02B — received evidence, CAL disposition

RI [PR #9](https://github.com/OpenRGD/rgd-reality-ingress/pull/9) remains draft, head 563d923cd65805a2fc40b431635241871720932f. Its owner return references [CI run 45813903783](https://github.com/OpenRGD/rgd-reality-ingress/actions/runs/45813903783) and reports 27 synthetic variants / 44 independently validated sidecars with the real pinned CAL exporter. These are RI's reported owner-side results, separate from the CAL CI cited above; no new CAL-side run against RI-A02B was performed in this consolidation.

**Proposed CAL position, for joint review:** agree in principle with native nonnegative exact integer range 0..2^64-1 for future versioned sequence, byte-count and monotonic-receipt fields, rejecting float/bool/string coercion and overflow in the native interface. Preserve the frozen v0.1 schema and historical records. JSON/JavaScript transport above 2^53-1 needs an explicitly lossless versioned representation or fail-closed rejection; never round or silently reinterpret it. Unicode code-point lengths and byte budgets are separate. The proposed helper is not an accepted wire contract, is not activated by this consolidation, and does not broaden CAL authority.

A02 correlates immutable bytes already supplied by authorized readers; A04 owns scoped metadata/payload access, consent, availability and retention; A05 owns stable verification. CAL consumes references and owns calibration profiles, revision and sidecar, not media lookup, history or physical identity. The nested RI ref and CAL sidecar remain separately validated. No changes to CAL code requested by the RI return.

## Gate to CAL-021-D

1. Receive and review the Console and Core returns and record exact owner PR/commit/evidence. Record decisions and any deferred conditions; no presumed approval.
2. Agree RI numeric profile and transport policy where required for D, keeping the current candidate interface off by default. Distinguish candidate-only agreement from production acceptance.
3. Build a reviewed cross-repository compatibility/migration matrix; run affected conformance suites against new owner candidates rather than overwriting original pins.
4. Only then proceed with the version/Registry/Console compatibility proposal. E still requires real installation, host/UI/offline/rollback acceptance; F remains a separate release decision.

**Freeze:** Registry retains org.openrgd.calibration@0.2.0 pinned to 81700401e2fa6d370a86b6c1f5a48802bc5f1fc7. No version bump, upstream edit, activation of ri-candidate, device access, motor privileges, or physical calibration approval. Existing records are not overwritten or resealed. No physical experiment is required for this documentation update.
