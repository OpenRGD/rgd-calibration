# Calibration milestones

## CAL-0.1 — Foundation (merged via PR #1)

- Read-only hardware-agnostic Rust contracts, profile lifecycle, evidence references, immutable revisions
- Integrity seals, schema, binding checks, 15 baseline tests, reproducible CI
- Canonical Core audit; no Core changes, drivers or active movement

## CAL-0.2 — Passive inspection (this PR)

- Explicit local Canonical Core JSON/JSONC ingestion, source provenance
- Joint/sensor calibration candidate inventory and missing-coverage report
- Cross-domain reference checks without implicit aliasing
- Separate nominal values / declared deltas / historical measurement claims
- Validate model/instance claims before *unapplied* scalar canonical delta preview
- JSON report schema + CLI and synthetic regression tests
- No motor commands, no sensor polling, no physical drift/certification claims

**Acceptance:** CI green, offline smoke CLI JSON, verified source inputs, zero changes to OpenRGD Canonical Core. All preview results must remain proposals.

## CAL-0.3 — Guided calibration

- Human-guided procedures for joints, cameras, IMU and other sensors
- Estimation/uncertainty, multi-measurement validation
- Attested external evidence and revision approval/rollback
- Vector-valued correction semantics and independently validated Canonical Core mapping

## CAL-0.4 — Safe active calibration

Only when an independently verified Somatic safety-gated execution protocol actually exists: bounded authorized micro-movements, e-stop and comms-loss rules, independent observation, guard checks and negative tests.

## CAL-0.5+ — Continuous calibration

Drift monitoring **from comparable timestamped physical measurements**, sensor fusion, versioned revalidation decisions, autonomous suggestions requiring validated evidence and explicit approval.
