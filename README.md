# OpenRGD Calibration

**CAL-0.1 — Foundation & Contracts (candidate)**

An independent, offline-first Rust library for versioned, instance-specific calibration records. It distinguishes the **ideal RGD robot model**, **physical observations**, **derived correction candidates**, and **validated revisions**.

## Current status

This repository is **non-actuating**. Opening a profile or loading this library **cannot move a robot**. There are no CAN/ROS2/Feetech/STM32/LeRobot drivers, no Somatic Runtime integration, and no active calibration. A profile marked as validated does **not** grant motion permission or prove physical safety.

The Canonical Core remains authoritative for the format of model-specific ideal values and calibration deltas. This repository does not alter it. In CAL-0.1, the Rust contract is a **candidate producer-side profile** and not yet an accepted OpenRGD standard.

## Build and test

```sh
cargo test --all-targets
```

Rust 2021; runs offline after Cargo dependencies have been installed or vendored. The library uses local JSON persistence, not a background service or database.

## Public Rust API

- `CalibrationProfile::new(...)` — create an uncalibrated instance-bound record.
- `validate(&profile)` — check structural invariants, traceability, versions, and physical evidence policies.
- `transition(&mut profile, next_state)` — checked lifecycle transitions.
- `seal(&mut profile)` / `verify(&profile)` — self-excluding SHA-256 content integrity.
- `check_bindings(...)` — reject a different physical instance, model hash, or registered device identity/firmware.
- `FileStore::save_new(...)` / `load(...)` — immutable JSON revisions; refuse overwrites.
- `PositionObservation::has_fresh_absolute_reference_claim(boot_id, now_ns, max_age_ns)` — check a **reboot-scoped and age-limited claim** using a monotonic clock; it does not authenticate sensors or authorize movement.
- `fork_for_recalibration(&parent, new_id, timestamp)` — begin a fresh, linked revision without silently reusing previous measurements or validations.

The persisted profile deliberately has **no current_position**: a position from a previous boot is not today's measured joint position. The safe rest pose is a separate reference, never the encoder zero by definition.

## Repository layout

```text
src/                 Rust public API, lifecycle, invariants, persistence
contracts/           Candidate JSON Schema contracts
docs/                Core audit and architecture boundaries
tests/               Offline failure and immutability tests
module.openrgd.json  Candidate Module Registry manifest (not yet installable)
.github/workflows/    CI
```

See [Canonical Core audit](docs/CORE_AUDIT.md), [architecture](docs/ARCHITECTURE.md), and [roadmap](docs/ROADMAP.md).

## Evidence and certification

The built-in validator checks linked source identifiers and a provenance **claim** (physical sensor or human measurement) before allowing a physical profile to carry the `validated` label. It does not independently authenticate a sensor, prove metrological correctness, validate safe behavior, or authorize the robot to actuate. Signed evidence, certified calibration, Chronograf authority, and safety-gate integration remain external responsibilities.

The JSON Schema is a wire format description; reference cross-checks and lifecycle rules are enforced in Rust. The profiles contain model-relative parameter references; mapping those to canonical `calibration.jsonc` fields is a separate adapter planned for CAL-0.2. 
