# CAL-0.1 — Architectural boundary and invariants

## Scope

```text
Ideal robot description (Canonical Core, read only)
          + physical instance identity and compatible hardware references
          + observation/evidence references (external, no invented samples)
                       ↓
          CalibrationProfile candidate
         measure → estimate → validate
                       ↓
       immutable revision + SHA-256 seal
                       ↓
      Console/Registry read-only consumer (future)
      Canonical Core calibration delta exporter (future)
      Runtime safety authority (external; never bypassed)
```

We represent the physical *delta* between theoretical model and measured body. The original description remains an independent, immutable input. The SHA-256 hash proves a record's content identity only; it does not prove its physical correctness.

## Lifecycle

The intended procedural flow is **Discover → Inspect → Prepare → Measure → Estimate → Validate → Commit → Monitor**.

The initial stored states are `uncalibrated` → `calibration_required` → `in_progress` → `validated`, with transitions to `degraded`, `expired` or `invalid`. A validated profile can be sent back to `calibration_required`. State transitions clear the previous integrity seal; the next edit must become a **new profile ID** linked through `previous_profile_id` and sealed before immutable storage. `fork_for_recalibration` creates a new `calibration_required` revision with new evidence/measurements/corrections empty; `FileStore::save_new` requires an intact, matching-instance predecessor when a link is provided. There is no mutation path in FileStore.

In CAL-0.1, `active` mode is reserved and rejected. The library has no motor command interfaces, callbacks, or discovery side effects.

## Observation, interpretation, and identity

- The profile records `physical_instance_id` separately from `model_ref` and `model_sha256`.
- Every measurement has `kind` (`observed` or `estimated`), a standard uncertainty, a unit, a model-relative parameter path, and evidence links.
- Each evidence item distinguishes physical sensor/human measurement from simulation, nominal model, or derived data, and may reference an external Chronon without asserting ownership of that ledger.
- Validation has a passing result, method, approver, timestamp, and evidence references. A physically `validated` record requires physical/human evidence in its validation report and correction references.
- This proves **structural provenance claims**, not that the sensor or human performed the measurement accurately. Trusted attestations and procedure certification are deferred.
- Sealing is deterministic JSON serialization with the `integrity_sha256` field set to null; the integrity format is `sha256:hex64`.
- `check_bindings` fails if the currently observed model, instance, or any declared hardware UID/firmware fingerprint mismatches the profile. Missing fields must not be filled from a guess.

## Seven different notions of zero

| Concept | Persisted or live? | Policy |
| --- | --- | --- |
| Encoder zero | Persisted reference definition | Not equal to model zero by default |
| Mechanical zero | Persisted reference definition | May be unknown |
| Kinematic/model zero | Persisted reference definition | Model-relative |
| Current observed joint position | **Live only** | Never inferred from previous session |
| Initial reference/start pose | Persisted definition, not current pose | Does not prove the robot is there |
| Safe rest pose | Persisted definition | Not automatically safe to enter or hold |
| Homing procedure | Optional external reference | Execution belongs to safety-gated runtime |

`PositionObservation` is ephemeral and reboot-scoped. Its helper requires a same-boot monotonic timestamp, limits how old that reading may be, and checks structural claims of absolute observability only; it cannot verify the hardware, establish metrological confidence, or authorize motion. If an absolute position cannot be evidenced **recently in the current boot**, it remains unknown.

## API / storage

The Rust API is used independently of the Console and has no network requirement for runtime operation. `FileStore` writes human-inspectable, sealed JSON under `profiles/<profile-id>.json` with a no-overwrite atomic persist. Revision history is linked, not rewritten; a previous profile can be retained and compared as evidence of drift.

The schema, Rust types, and Registry manifest are local candidates. No claim is made that the current Console or Module Registry implements the manifest format.

## Deferred integrations

- **CAL-0.2**: explicit mapping to canonical delta fields, passive read and completeness reports
- **CAL-0.3**: guided measurements, estimation algorithms and validation procedures
- **CAL-0.4**: active micro-movement requests **only** via an existing verified Somatic Runtime safety-gate interface with consent, bounds, stop, and communication-loss guarantees
- **CAL-0.5+**: drift evidence, recalibration rules and cognitive suggestions; no autonomous commit without independent validation

No component of CAL-0.1 may turn a persisted calibration record into runtime motion authority.
