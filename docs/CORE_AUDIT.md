# CAL-0.1 — Canonical Core reconnaissance (2026-10-19)

Source of authority: [OpenRGD/openrgd](https://github.com/OpenRGD/openrgd) default branch `main`, checked in preparation for CAL-0.1. The authoritative editable model is under `spec/`; `standard/` is the strict JSON leaf mirror. OpenRGD standard bundle is 0.2.0 draft, and version/maturity of domains must be kept separate.

## Existing primitives: reuse instead of duplicating

| Core file | Relevant fields | Calibration's responsibility |
| --- | --- | --- |
| [calibration.jsonc](https://github.com/OpenRGD/openrgd/blob/main/spec/01_foundation/calibration.jsonc) | `meta_group`, `target_description_hash_str`, `kinematic_offsets_map`, `sensor_extrinsics_delta_map`, `sensor_intrinsics_map`, `dynamics_correction_map` | Produce versioned, supported **instance deltas**, never modify ideal source |
| [description.jsonc](https://github.com/OpenRGD/openrgd/blob/main/spec/01_foundation/description.jsonc) | `model_id_str`, `joint_topology_map`, `sensors_mounting_map`, `actuators_topology_map` | Reference the immutable ideal model and stable joint/sensor IDs |
| [actuation_topology.jsonc](https://github.com/OpenRGD/openrgd/blob/main/spec/01_foundation/actuation_topology.jsonc) | `joint_actuator_mapping_map`, `control_profiles_map` | Identify target joints and applications, not command actuators |
| [actuation_dynamics.jsonc](https://github.com/OpenRGD/openrgd/blob/main/spec/01_foundation/actuation_dynamics.jsonc) | `actuator_model_dynamics_map`, `joint_dynamics_map`, `safety_envelopes_map` | Distinguish nominal dynamics from measured corrections |
| [hal_mapping.jsonc](https://github.com/OpenRGD/openrgd/blob/main/spec/01_foundation/hal_mapping.jsonc) | `hardware_uid_str`, `firmware_ref_str`, logical joint/sensor mappings | Bind revisions to observed device identities without depending on drivers |
| [sensor_fidelity.jsonc](https://github.com/OpenRGD/openrgd/blob/main/spec/01_foundation/sensor_fidelity.jsonc) | `sensor_error_profiles_map`, noise/bias/fault injection | Never mistake simulation noise distributions for real observations |
| [runtime_validation.jsonc](https://github.com/OpenRGD/openrgd/blob/main/spec/02_operation/runtime_validation.jsonc) | `calibration_validity` pre-flight, max-age and model-hash checks | Report applicability and degradation; **not** decide if motion is permitted |
| [safety_critical.jsonc](https://github.com/OpenRGD/openrgd/blob/main/spec/02_operation/safety_critical.jsonc) | Physical caps, reflex triggers, E-stop | Stay outside the safety-critical enforcement authority |

Related: [Agent candidate contracts](https://github.com/OpenRGD/openrgd/tree/main/contracts/agent/v0.1.0) distinguish the Somatic Translator, Operation Safety Gate, Body Adapter, and append-only Chronon evidence. Their `STATUS.json` says **candidate**, **non-normative**, **not accepted**. No dependency on these contracts is mandatory for CAL-0.1.

## Material gaps

1. The example calibration file stores deltas but does not define immutable evidence, per-value measurement uncertainty, correction approval, a complete lifecycle, compatibility after device change, or rollback history.
2. Encoder zero, mechanical zero, model zero, start pose, safe rest, current position, and homing have no single, explicitly separated persisted/ephemeral contract.
3. There is no accepted Module Registry contract in the reviewed Canonical Core; `module.openrgd.json` is only a proposal.
4. A canonical Chronon writer or Somatic safety-gated active-calibration API cannot be assumed to exist in this repository.

## Conflicts to resolve separately (do not modify Canonical Core here)

- `runtime_validation.jsonc` marks `calibration_validity` as `MAJOR` while its individual failure policy says `block_motion_bool: false`; the global `MAJOR` policy says `allow_motion_bool: false`. Runtime authority must resolve this; this module will **not** silently pick one.
- The example `actuation_topology.jsonc` maps a knee to `leg_right_knee_pitch`, while `description.jsonc` uses `knee_joint_right`. An instance mapping/alias is necessary; never assume IDs are interchangeable.
- `sensor_fidelity.jsonc` contains `right_wrist_ft` but that sensor is absent from the current `description.jsonc` example. Validate references rather than fabricate missing sensors.
- `sensor_fidelity.jsonc` links runtime validation under `01_foundation/runtime_validation.jsonc`, but the file is under `02_operation/`.
- The model's example `calibration.jsonc` uses a SHA-256 value beginning `e3b0c442...`, the well-known hash of an empty byte string. Do not treat seed/reference values as validated physical data.
- Canonical sample fields and comments discuss standards compliance; no safety certification is established by this audit. Test fixtures, nominal values, and textual safety claims must not be mistaken for hardware approval.

## Version boundary decision

CAL-0.1 defines its **own** instance measurement/evidence profile as a candidate. It does **not** redefine the 01_foundation calibration-domain schema. Export/conversion to the Canonical Core's nested delta maps requires explicit, independently tested field mapping and provenance enforcement in a later milestone.
