# OpenRGD Calibration

**CAL-0.2 — Passive Calibration (candidate)** • **CAL-0.1 — Foundation merged**

An independent, offline-first Rust module for calibration profile provenance, metrology records, and **read-only inspection** of an OpenRGD robot model. It never connects to actuators and never writes corrections into the Canonical Core. Calibration is not homing, and loading a description cannot move a robot.

## Development roadmap

The **[Calibration Master Development Checklist](docs/CALIBRATION_MASTER_CHECKLIST.md)** is the source of truth for completed milestones, next steps, quality gates and cross-repository dependencies. CAL-0.1/0.2 are merged and CI-verified; guided sessions, host-level Console acceptance, active calibration and continuous drift monitoring are not yet complete.

## Quick start

```bash
cargo test --all-targets --locked

# Inspect a locally cloned canonical OpenRGD repository:
cargo run --locked --bin rgd-calibration -- inspect --bundle /path/to/openrgd

# Generate a local report WITHOUT overwriting an existing file:
cargo run --locked --bin rgd-calibration -- inspect \
  --bundle /path/to/openrgd \
  --out ./calibration-passive-report.json
```

The command looks for `spec/01_foundation/description.jsonc` first (canonical authoring source), then `standard/01_foundation/description.json`, then a direct `01_foundation` directory. It reads optional `calibration`, `actuation_topology`, `hal_mapping`, and `sensor_fidelity` documents from the **same** directory, not from another version or an online source. No file or motor is changed. Strict JSON and JSONC comments/trailing commas are supported.

## What you get

- Model and source-file raw-byte SHA-256 digests (distinct from OpenRGD source-tree/model identity hashes).
- Per-joint zero-offset/backlash candidates; sensor extrinsic candidates; camera intrinsic candidates when applicable.
- Separate **nominal**, **declared calibration**, and **saved historical measurement** fields.
- Missing/declared/recorded coverage, orphan IDs, unresolved references, invalid shapes, and unsupported mappings.
- Explicit `current_robot_position_observed: false`, `physical_drift_evaluated: false`, and `claims_physical_calibration_certified: false`.
- Optional **unapplied delta preview**, only from a sealed `validated`-state profile with explicit caller-supplied instance/model bindings. Even then the hardware is **not verified**.

```bash
cargo run --locked --bin rgd-calibration -- inspect \
  --bundle /path/to/openrgd \
  --profile /path/to/calibration-profile.json \
  --instance THE_ACTUAL_ROBOT_INSTANCE_ID \
  --model-hash sha256:THE_EXTERNALLY_VERIFIED_MODEL_DIGEST
```

**Important:** `--model-hash` is an external model digest established by the caller, **not** the SHA-256 of an arbitrarily selected file. Matching these two CLI strings is a structural assertion, not sensor/device authentication. Never interpret the resulting preview as permission to move.

## Stability / governance

- `src/model.rs`, `src/validation.rs`, `src/store.rs`: CAL-0.1 contracts (schema version `0.1.0`), integrity, lifecycle, immutable profile storage.
- `src/source.rs`: bounded, local JSON/JSONC reader.
- `src/passive.rs`: candidate CAL-0.2 coverage and consistency report. Contract: `contracts/passive-report.v0.2.schema.json`.
- `src/bin/rgd-calibration.rs`: explicit operator-invoked offline CLI.
- `module.openrgd.json`: **proposal-only** module manifest. This does not imply a functional Module Registry.
- `tests/`: synthetic tests only, never physical evidence.
- `docs/PASSIVE_CALIBRATION.md`: limitations, reconciliation and mapping details.

Neither profile hash integrity nor a source's claim of `validated` implies a valid physical calibration, human/sensor certification, a safe resting pose, or authorization by Somatic Runtime. We have **no automatic motor control, telemetry or homing** in CAL-0.2.

See [audit](docs/CORE_AUDIT.md), [architecture](docs/ARCHITECTURE.md), [CAL-0.2 details](docs/PASSIVE_CALIBRATION.md) and [roadmap](docs/ROADMAP.md). All changes to `OpenRGD/openrgd` are out of scope.
