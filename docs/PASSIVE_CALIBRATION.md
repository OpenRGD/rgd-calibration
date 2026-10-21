# CAL-0.2 — Passive Inspection Contract (candidate)

## Core reading

`BundleSource::load(root)` supports:
1. `<root>/spec/01_foundation/description.jsonc` (preferred);
2. `<root>/standard/01_foundation/description.json` (strict mirror fallback);
3. `<root>/01_foundation/description.json[c]`;
4. directly supplied `01_foundation` files.

All optional domain documents are read from the **same** directory as the selected description; no mixed-version fallback. Reads reject non-regular files/symlinks and >4 MiB content. The operator explicitly selects the root. No auto-discovery of hardware, no network calls, no implicit report persistence.

JSONC supports comments/trailing commas without deleting comment-shaped strings. JSON numeric format is not proof of units or physical plausibility; unsupported fields are left unresolved. The source-file SHA-256 is the **byte identity of the particular loaded file**, not the Canonical Core's `OPENRGD_SOURCE_TREE_SHA256_V1` digest, nor a certificate.

## Per-component inventory

| Nominal entity | Candidate canonical calibration paths | Meaning |
| --- | --- | --- |
| `joint_topology_map.<id>` | `kinematic_offsets_map.<id>.zero_point_offset_rad_float`, `backlash_hysteresis_rad_float` | Joint zero / mechanical backlash candidates; NOT a currently measured pose |
| `sensors_mounting_map.<id>` (with ideal pose) | `sensor_extrinsics_delta_map.<id>.mount_delta_xyz_m_vec3_float`, `mount_delta_rpy_rad_vec3_float` | Separate ideal pose from physical extrinsic delta |
| Camera sensor | `sensor_intrinsics_map.<id>.camera_matrix_k_vec9_float`, `distortion_coeffs_vec5_float`, `distortion_model_enum` | Camera-specific intrinsics, not mandatory for every sensor |

`coverage` is **candidate field inventory completeness**, not a pass/fail metrological assessment. Fields absent from the canonical calibration document are not invented, nor is a sample value automatically promoted to a measured observation. Historical measurements from a sealed profile remain historical claims. We intentionally do not set a global ratio of physical calibration quality: applicability, geometry, dynamic bounds and uncertainty must be reviewed per component.

## Cross-domain consistency

- `actuation_topology.joint_actuator_mapping_map[*].target_joint_ref_str` is checked against the description's exact joint IDs.
- `description.actuators_topology_map[*].drives_joint_ref_str` is checked against exact joint IDs.
- `hal_mapping.actuator_drivers_map[*].logical_actuator_ref_str` is checked against description actuator IDs.
- `hal_mapping.sensor_drivers_map[*].logical_sensor_ref_str` and `sensor_fidelity.sensor_error_profiles_map` are checked against description sensor IDs.
- Orphan keys under `calibration.kinematic_offsets_map`, `sensor_extrinsics_delta_map`, `sensor_intrinsics_map`, and `dynamics_correction_map` are flagged. `global_body` is a recognized non-joint key for dynamics.
- ID mismatches are findings; **no fuzzy names, no inferred aliases, no modification of the files.**

The BHL reference example currently contains mismatches (e.g. `knee_joint_right` vs `leg_right_knee_pitch`) that require a separately governed name mapping. The model is illustrative, not a physically calibrated Paky-01.

## Identity and proposed delta preview

Input: optional `CalibrationProfile` from CAL-0.1 (must pass `verify`), plus optional `InspectionContext` from the caller. A profile's `validated` state is a **provenance/approval claim**, not a verified robot. To render a preview, CAL-0.2 currently requires:
- `profile.state == validated` with intact integrity seal and evidence links;
- caller-supplied `physical_instance_id` and authoritative `model_sha256` exactly match the profile;
- each scalar correction has an explicitly supported canonical field, joint ID, unit and operation;
- a saved scalar measurement with the same target path and unit is required before that correction can be previewed; arrays and enums from the Canonical Core cannot be inferred from CAL-0.1 scalar measurements.

Only currently supported scalar preview paths:

| Target | Operation | Unit | Additional rules |
| --- | --- | --- | --- |
| `kinematic_offsets_map.<joint>.zero_point_offset_rad_float` | `additive_delta` | `rad` | Joint must exist |
| `kinematic_offsets_map.<joint>.backlash_hysteresis_rad_float` | `absolute_override` | `rad` | Value >=0 |
| `dynamics_correction_map.<joint>.friction_scale_factor_float` | `multiplicative_factor` | `1` | Value >0 |
| `dynamics_correction_map.<joint>.damping_scale_factor_float` | `multiplicative_factor` | `1` | Value >0 |

The exported **patch preview** is explicitly `applied:false` and `hardware_binding_verified:false`. No vector correction export is supported by the v0.1 scalar-only Correction struct; CAL-0.3 must define that separately. Only a controlled external producer could later merge a proposed patch into `01_foundation/calibration` after model-specific review. This module never writes that file, nor uses this state as runtime safety authority.

## What CAL-0.2 CANNOT tell you

- Where the physical robot is now, whether an incremental encoder survived reboot, or whether it can return safely to its resting pose.
- Whether historical `EvidenceOrigin::PhysicalSensor` or `HumanMeasurement` is honest, accurate, signed or currently observed.
- Whether a camera/IMU/joint is in tolerance: a complete acceptance procedure and trusted observations are still needed.
- Whether physical drift occurred; a delta between two declarations is not comparable, thresholded physical time-series evidence.
- Whether it is safe to move; only a separately implemented Somatic Runtime/Safety Gate can ever decide this.

An optional `--out` writes a report *only* to a caller-selected **new** path (no overwrite); without it, JSON goes to stdout. Tests use intentionally synthetic fixtures.
