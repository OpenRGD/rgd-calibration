# Calibration milestones

## CAL-0.1 — Foundation (this PR)

- Core reconnaissance and contradiction log
- Independent Rust profile / measurement / evidence / hardware-binding contracts
- Explicit lifecycle and reject-active policy
- JSON Schema 2020-12 candidate contracts
- Self-excluding SHA-256 seal and immutable offline filesystem store
- Fail-closed applicability to model/instance/devices
- Reboot-scoped ephemeral position-reference contract
- CI with offline unit tests and JSON syntax check
- Proposal-only Module Registry manifest

**Acceptance:** source builds, all tests pass on GitHub CI, no motor/driver dependency, no modification to Canonical Core.

## CAL-0.2 — Passive calibration

- Canonical Core model loader and field-reference resolver
- Cross-domain reference normalization and coverage/missing-data reports
- Read-only export preview to the existing calibration delta maps
- Drift and incompatibility reporting without movement

## CAL-0.3 — Guided calibration

- Human-guided procedures for joints, cameras, IMU and other sensors
- Estimation/uncertainty, multi-measurement validation
- Attested external evidence and revision approval/rollback

## CAL-0.4 — Safe active calibration

Only when an independently verified Somatic safety-gated execution protocol actually exists: bounded authorized micro-movements, e-stop and comms-loss rules, independent observation, guard checks and negative tests.

## CAL-0.5+ — Continuous calibration

Drift monitoring, sensor fusion, versioned revalidation decisions, autonomous suggestions requiring validated evidence and explicit approval.
