//! Passive calibration inventory, consistency findings, and read-only delta
//! proposals. Every input is a local document or an explicitly supplied,
//! sealed historical profile. No device is observed or commanded here.
use crate::model::{CalibrationProfile, CalibrationState, CorrectionOperation, Measurement};
use crate::source::{BundleSource, SourceDocument};
use crate::validation::{verify, CalibrationError};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const PASSIVE_REPORT_VERSION: &str = "0.2.0";

#[derive(Clone, Debug, Default)]
pub struct InspectionContext {
    /// Caller assertion only; this module does NOT authenticate a robot.
    pub physical_instance_id: Option<String>,
    /// Authoritatively established model digest supplied by the caller. This
    /// is NOT the per-file raw byte digest shown in source_digests.
    pub model_sha256: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingStatus {
    NoProfile,
    MissingContext,
    ClaimedMatch,
    Mismatch,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageKind {
    Absent,
    DeclaredOnly,
    SavedEvidence,
}

#[derive(Clone, Debug, Serialize)]
pub struct SourceDigest {
    pub role: String,
    pub path: String,
    pub raw_file_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Information,
    Warning,
}

#[derive(Clone, Debug, Serialize)]
pub struct Finding {
    pub severity: Severity,
    pub code: String,
    pub source: String,
    pub reference: String,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CandidateField {
    pub parameter_ref: String,
    pub component_kind: String,
    pub component_id: String,
    pub unit: String,
    /// Optional model value for reference only. This is not a measured value.
    pub nominal_ref: Option<String>,
    pub nominal_value: Option<Value>,
    /// Claimed values inside a Canonical Core calibration file are NOT evidence.
    pub declared_value: Option<Value>,
    /// Historical saved measurement claims (never current sensor reads).
    pub stored_measurements: Vec<Measurement>,
    pub coverage: CoverageKind,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CoverageCounts {
    pub candidate_fields: usize,
    pub declared_fields: usize,
    pub saved_evidence_fields: usize,
    pub without_saved_measurements: usize,
    pub without_declared_values: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProposedDelta {
    pub target: String,
    pub operation: CorrectionOperation,
    pub value: f64,
    pub unit: String,
    pub physical_verification_required: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PreviewPatch {
    pub status: String,
    pub applied: bool,
    pub hardware_binding_verified: bool,
    /// This contains ONLY the proposed fields, not an RGD replacement bundle.
    pub canonical_calibration_delta_patch: Value,
    pub proposed_deltas: Vec<ProposedDelta>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PassiveReport {
    pub report_version: String,
    pub model_id: String,
    pub source_digests: Vec<SourceDigest>,
    pub profile_id: Option<String>,
    pub profile_state: Option<CalibrationState>,
    pub profile_binding: BindingStatus,
    pub claims_physical_calibration_certified: bool,
    pub current_robot_position_observed: bool,
    pub physical_drift_evaluated: bool,
    pub coverage: CoverageCounts,
    pub parameters: Vec<CandidateField>,
    pub findings: Vec<Finding>,
    pub export_preview: Option<PreviewPatch>,
}

fn finding(
    out: &mut Vec<Finding>,
    severity: Severity,
    code: &str,
    source: &str,
    reference: &str,
    detail: &str,
) {
    out.push(Finding {
        severity,
        code: code.to_owned(),
        source: source.to_owned(),
        reference: reference.to_owned(),
        detail: detail.to_owned(),
    });
}

fn map<'a>(doc: &'a Value, name: &str) -> Option<&'a Map<String, Value>> {
    doc.get(name).and_then(Value::as_object)
}

fn logical_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn source_paths(bundle: &BundleSource) -> Vec<SourceDigest> {
    [
        Some(&bundle.description),
        bundle.calibration.as_ref(),
        bundle.actuation_topology.as_ref(),
        bundle.hal_mapping.as_ref(),
        bundle.sensor_fidelity.as_ref(),
    ]
    .into_iter()
    .flatten()
    .map(|source| SourceDigest {
        role: source.role.to_owned(),
        path: source.path.clone(),
        raw_file_sha256: source.raw_sha256.clone(),
    })
    .collect()
}

fn declared_field(
    calibration: Option<&SourceDocument>,
    section: &str,
    component_id: &str,
    field: &str,
) -> Option<Value> {
    calibration?
        .value
        .get(section)?
        .get(component_id)?
        .get(field)
        .cloned()
}

fn valid_shape(value: &Value, expected: usize, enum_string: bool) -> bool {
    if enum_string {
        return value.as_str().is_some_and(|v| !v.is_empty());
    }
    if expected == 1 {
        return value.is_number();
    }
    value
        .as_array()
        .is_some_and(|items| items.len() == expected && items.iter().all(Value::is_number))
}

#[derive(Clone, Copy)]
struct FieldDefinition<'a> {
    section: &'a str,
    key: &'a str,
    unit: &'a str,
    length: usize,
    as_enum: bool,
}

fn register(
    out: &mut Vec<CandidateField>,
    findings: &mut Vec<Finding>,
    bundle: &BundleSource,
    kind: &str,
    id: &str,
    spec: FieldDefinition<'_>,
    nominal: Option<(&str, &Value)>,
    measurements: &BTreeMap<&str, Vec<Measurement>>,
) {
    let param_ref = format!("{}.{}.{}", spec.section, id, spec.key);
    let raw = declared_field(bundle.calibration.as_ref(), spec.section, id, spec.key);
    if raw
        .as_ref()
        .is_some_and(|val| !valid_shape(val, spec.length, spec.as_enum))
    {
        finding(
            findings,
            Severity::Warning,
            "invalid_declared_shape",
            "calibration",
            &param_ref,
            "Declared field has an unexpected shape; it is excluded, not repaired.",
        );
    }
    let declared = raw.filter(|val| valid_shape(val, spec.length, spec.as_enum));
    let saved = measurements
        .get(param_ref.as_str())
        .cloned()
        .unwrap_or_default();
    let coverage = if !saved.is_empty() {
        CoverageKind::SavedEvidence
    } else if declared.is_some() {
        CoverageKind::DeclaredOnly
    } else {
        CoverageKind::Absent
    };
    out.push(CandidateField {
        parameter_ref: param_ref,
        component_kind: kind.to_owned(),
        component_id: id.to_owned(),
        unit: spec.unit.to_owned(),
        nominal_ref: nominal.map(|(name, _)| name.to_owned()),
        nominal_value: nominal.map(|(_, value)| value.clone()),
        declared_value: declared,
        stored_measurements: saved,
        coverage,
    });
}

fn verify_reference_map(
    findings: &mut Vec<Finding>,
    source: &str,
    json: &Value,
    section: &str,
    field: &str,
    valid_ids: &BTreeSet<&str>,
) {
    if let Some(objects) = map(json, section) {
        for (alias, entry) in objects {
            if let Some(target) = entry.get(field).and_then(Value::as_str) {
                if !valid_ids.contains(target) {
                    finding(
                        findings,
                        Severity::Warning,
                        "unresolved_cross_domain_reference",
                        source,
                        &format!("{section}.{alias}.{field}"),
                        &format!(
                            "Target '{target}' is not declared by the ideal description; no alias is inferred."
                        ),
                    );
                }
            } else {
                finding(
                    findings,
                    Severity::Warning,
                    "missing_cross_domain_reference",
                    source,
                    &format!("{section}.{alias}.{field}"),
                    "Expected a string logical reference.",
                );
            }
        }
    }
}

fn orphan_keys(
    findings: &mut Vec<Finding>,
    source: &str,
    value: &Value,
    section: &str,
    keys: &BTreeSet<&str>,
    allow_global_body: bool,
) {
    if let Some(entries) = map(value, section) {
        for key in entries.keys() {
            if !(keys.contains(key.as_str()) || (allow_global_body && key == "global_body")) {
                finding(
                    findings,
                    Severity::Warning,
                    "orphan_component_reference",
                    source,
                    &format!("{section}.{key}"),
                    "No matching component in description; do not assume a name alias.",
                );
            }
        }
    }
}

/// Model-reference equality is strict at the logical file level. A .jsonc source
/// and its tracked strict .json mirror are equivalent *references*, NOT hashes.
fn same_description_ref(ref_name: &str) -> bool {
    matches!(
        ref_name,
        "01_foundation/description.jsonc" | "01_foundation/description.json"
    )
}

fn report_binding(
    profile: Option<&CalibrationProfile>,
    context: &InspectionContext,
    findings: &mut Vec<Finding>,
) -> BindingStatus {
    let Some(profile) = profile else {
        return BindingStatus::NoProfile;
    };
    if !same_description_ref(&profile.model_ref) {
        finding(
            findings,
            Severity::Warning,
            "model_reference_unrecognized",
            "profile",
            &profile.model_ref,
            "CAL-0.2 supports the canonical 01_foundation/description logical reference only.",
        );
        return BindingStatus::Mismatch;
    }
    let (Some(instance), Some(model_hash)) = (
        context.physical_instance_id.as_ref(),
        context.model_sha256.as_ref(),
    ) else {
        finding(
            findings,
            Severity::Warning,
            "missing_binding_context",
            "profile",
            &profile.profile_id,
            "An explicit physical instance ID and authoritative model digest are needed for candidate export. The raw source file hash is NOT a replacement.",
        );
        return BindingStatus::MissingContext;
    };
    if instance != &profile.physical_instance_id || model_hash != &profile.model_sha256 {
        finding(
            findings,
            Severity::Warning,
            "profile_binding_mismatch",
            "profile",
            &profile.profile_id,
            "Declared physical instance or model digest does not match the supplied context.",
        );
        return BindingStatus::Mismatch;
    }
    finding(
        findings,
        Severity::Information,
        "caller_asserted_binding_only",
        "profile",
        &profile.profile_id,
        "Instance/model identity matches caller claims; no live hardware identity or attestation has been checked.",
    );
    BindingStatus::ClaimedMatch
}

fn build_preview(
    profile: Option<&CalibrationProfile>,
    binding: &BindingStatus,
    joint_ids: &BTreeSet<&str>,
    findings: &mut Vec<Finding>,
) -> Option<PreviewPatch> {
    let profile = profile?;
    if profile.state != CalibrationState::Validated || *binding != BindingStatus::ClaimedMatch {
        return None;
    }
    let mut patch = Map::new();
    let mut changes = Vec::new();
    for correction in &profile.corrections {
        let pieces: Vec<&str> = correction.parameter_ref.split('.').collect();
        if pieces.len() != 3 {
            finding(
                findings,
                Severity::Warning,
                "unsupported_preview_path",
                "profile",
                &correction.parameter_ref,
                "Only explicit three-segment canonical calibration fields can be previewed.",
            );
            continue;
        }
        let supported = match (
            pieces[0],
            pieces[2],
            correction.operation,
            correction.unit.as_str(),
        ) {
            (
                "kinematic_offsets_map",
                "zero_point_offset_rad_float",
                CorrectionOperation::AdditiveDelta,
                "rad",
            ) => true,
            (
                "kinematic_offsets_map",
                "backlash_hysteresis_rad_float",
                CorrectionOperation::AbsoluteOverride,
                "rad",
            ) => correction.value >= 0.0,
            (
                "dynamics_correction_map",
                "friction_scale_factor_float",
                CorrectionOperation::MultiplicativeFactor,
                "1",
            ) => correction.value > 0.0,
            (
                "dynamics_correction_map",
                "damping_scale_factor_float",
                CorrectionOperation::MultiplicativeFactor,
                "1",
            ) => correction.value > 0.0,
            _ => false,
        };
        if !supported || !joint_ids.contains(pieces[1]) {
            finding(
                findings,
                Severity::Warning,
                "unsupported_preview_mapping",
                "profile",
                &correction.parameter_ref,
                "Unknown joint/operation/unit or non-scalar target: proposal intentionally omitted.",
            );
            continue;
        }
        let section = patch
            .entry(pieces[0].to_owned())
            .or_insert_with(|| Value::Object(Map::new()));
        let joints = section.as_object_mut().expect("section created as object");
        let joint = joints
            .entry(pieces[1].to_owned())
            .or_insert_with(|| Value::Object(Map::new()));
        joint
            .as_object_mut()
            .expect("joint created as object")
            .insert(pieces[2].to_owned(), json!(correction.value));
        changes.push(ProposedDelta {
            target: correction.parameter_ref.clone(),
            operation: correction.operation,
            value: correction.value,
            unit: correction.unit.clone(),
            physical_verification_required: true,
        });
    }
    if changes.is_empty() {
        return None;
    }
    Some(PreviewPatch {
        status: "candidate_only_no_auto_commit".into(),
        applied: false,
        hardware_binding_verified: false,
        canonical_calibration_delta_patch: Value::Object(patch),
        proposed_deltas: changes,
    })
}

/// Read-only inspection of a local OpenRGD model against optional declared
/// calibration values and a sealed, historical, instance-specific profile.
/// No external identity, sensor, timing, or motor authority is assumed.
pub fn inspect_passive(
    bundle: &BundleSource,
    profile: Option<&CalibrationProfile>,
    context: &InspectionContext,
) -> Result<PassiveReport, CalibrationError> {
    if let Some(profile) = profile {
        verify(profile)?;
    }
    let description = &bundle.description.value;
    let model_id = description
        .pointer("/meta_group/model_id_str")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            CalibrationError::Invalid("description.meta_group.model_id_str missing".into())
        })?;
    let joints = map(description, "joint_topology_map").ok_or_else(|| {
        CalibrationError::Invalid("description.joint_topology_map must be an object".into())
    })?;
    let sensors = map(description, "sensors_mounting_map").ok_or_else(|| {
        CalibrationError::Invalid("description.sensors_mounting_map must be an object".into())
    })?;
    let joint_ids: BTreeSet<&str> = joints.keys().map(String::as_str).collect();
    let sensor_ids: BTreeSet<&str> = sensors.keys().map(String::as_str).collect();
    let actuators = map(description, "actuators_topology_map");
    let actuator_ids: BTreeSet<&str> = actuators
        .map(|mapping| mapping.keys().map(String::as_str).collect())
        .unwrap_or_default();

    let mut findings = Vec::new();
    if bundle.calibration.is_none() {
        finding(
            &mut findings,
            Severity::Information,
            "optional_source_not_provided",
            "bundle",
            "calibration",
            "No calibration file was present; no correction values are assumed.",
        );
    }
    let mut stored: BTreeMap<&str, Vec<Measurement>> = BTreeMap::new();
    if let Some(profile) = profile {
        for measurement in &profile.measurements {
            stored
                .entry(&measurement.parameter_ref)
                .or_default()
                .push(measurement.clone());
        }
        if profile.state != CalibrationState::Validated {
            finding(
                &mut findings,
                Severity::Warning,
                "profile_not_validated",
                "profile",
                &profile.profile_id,
                "Saved measurement records are not approved calibration corrections.",
            );
        }
        if !profile.hardware.is_empty() {
            finding(
                &mut findings,
                Severity::Information,
                "hardware_identity_not_observed",
                "profile",
                &profile.profile_id,
                "Hardware UID/firmware in a file are references, not current physical discovery.",
            );
        }
    }

    let mut parameters = Vec::new();
    for (id, _entry) in joints {
        if !logical_id(id) {
            finding(
                &mut findings,
                Severity::Warning,
                "invalid_component_identifier",
                "description",
                id,
                "Only lowercase ASCII letters, digits, underscore are supported for dotted paths.",
            );
            continue;
        }
        for field in [
            "zero_point_offset_rad_float",
            "backlash_hysteresis_rad_float",
        ] {
            register(
                &mut parameters,
                &mut findings,
                bundle,
                "joint",
                id,
                FieldDefinition {
                    section: "kinematic_offsets_map",
                    key: field,
                    unit: "rad",
                    length: 1,
                    as_enum: false,
                },
                None,
                &stored,
            );
        }
    }
    for (id, entry) in sensors {
        if !logical_id(id) {
            finding(
                &mut findings,
                Severity::Warning,
                "invalid_component_identifier",
                "description",
                id,
                "Only lowercase ASCII letters, digits, underscore are supported for dotted paths.",
            );
            continue;
        }
        for (field, nominal_key, unit) in [
            (
                "mount_delta_xyz_m_vec3_float",
                "mount_pose_xyz_m_vec3_float",
                "m",
            ),
            (
                "mount_delta_rpy_rad_vec3_float",
                "mount_pose_rpy_rad_vec3_float",
                "rad",
            ),
        ] {
            if let Some(nominal) = entry.get(nominal_key) {
                let reference = format!("sensors_mounting_map.{id}.{nominal_key}");
                register(
                    &mut parameters,
                    &mut findings,
                    bundle,
                    "sensor",
                    id,
                    FieldDefinition {
                        section: "sensor_extrinsics_delta_map",
                        key: field,
                        unit,
                        length: 3,
                        as_enum: false,
                    },
                    Some((&reference, nominal)),
                    &stored,
                );
            }
        }
        let is_camera = entry
            .get("device_type_enum")
            .and_then(Value::as_str)
            .is_some_and(|kind| kind.contains("CAMERA"));
        if is_camera {
            for (field, arity) in [
                ("camera_matrix_k_vec9_float", 9),
                ("distortion_coeffs_vec5_float", 5),
            ] {
                register(
                    &mut parameters,
                    &mut findings,
                    bundle,
                    "sensor",
                    id,
                    FieldDefinition {
                        section: "sensor_intrinsics_map",
                        key: field,
                        unit: "camera_model",
                        length: arity,
                        as_enum: false,
                    },
                    None,
                    &stored,
                );
            }
            register(
                &mut parameters,
                &mut findings,
                bundle,
                "sensor",
                id,
                FieldDefinition {
                    section: "sensor_intrinsics_map",
                    key: "distortion_model_enum",
                    unit: "enum",
                    length: 1,
                    as_enum: true,
                },
                None,
                &stored,
            );
        }
    }
    if let Some(calibration) = &bundle.calibration {
        for (name, keys, allow_global) in [
            ("kinematic_offsets_map", &joint_ids, false),
            ("dynamics_correction_map", &joint_ids, true),
            ("sensor_extrinsics_delta_map", &sensor_ids, false),
            ("sensor_intrinsics_map", &sensor_ids, false),
        ] {
            orphan_keys(
                &mut findings,
                "calibration",
                &calibration.value,
                name,
                keys,
                allow_global,
            );
        }
    }
    if let Some(actuation_topology) = &bundle.actuation_topology {
        verify_reference_map(
            &mut findings,
            "actuation_topology",
            &actuation_topology.value,
            "joint_actuator_mapping_map",
            "target_joint_ref_str",
            &joint_ids,
        );
    }
    if let Some(hal) = &bundle.hal_mapping {
        verify_reference_map(
            &mut findings,
            "hal_mapping",
            &hal.value,
            "actuator_drivers_map",
            "logical_actuator_ref_str",
            &actuator_ids,
        );
        verify_reference_map(
            &mut findings,
            "hal_mapping",
            &hal.value,
            "sensor_drivers_map",
            "logical_sensor_ref_str",
            &sensor_ids,
        );
    }
    if let Some(fidelity) = &bundle.sensor_fidelity {
        orphan_keys(
            &mut findings,
            "sensor_fidelity",
            &fidelity.value,
            "sensor_error_profiles_map",
            &sensor_ids,
            false,
        );
    }
    // A model-declared actuation mapping must resolve; missing or aliased
    // IDs are not silently normalized to a coincidentally similar name.
    verify_reference_map(
        &mut findings,
        "description",
        description,
        "actuators_topology_map",
        "drives_joint_ref_str",
        &joint_ids,
    );

    let known: BTreeSet<&str> = parameters
        .iter()
        .map(|p| p.parameter_ref.as_str())
        .collect();
    for path in stored.keys() {
        if !known.contains(path) {
            finding(
                &mut findings,
                Severity::Warning,
                "unmapped_stored_measurement",
                "profile",
                path,
                "Measurement has no recognized field in the selected model; it remains historical only.",
            );
        }
    }
    let binding = report_binding(profile, context, &mut findings);
    let mut counts = CoverageCounts::default();
    for param in &parameters {
        counts.candidate_fields += 1;
        if param.declared_value.is_some() {
            counts.declared_fields += 1;
        } else {
            counts.without_declared_values += 1;
        }
        if param.stored_measurements.is_empty() {
            counts.without_saved_measurements += 1;
        } else {
            counts.saved_evidence_fields += 1;
        }
    }
    if let (Some(calibration), Some(profile)) = (&bundle.calibration, profile) {
        if profile.state == CalibrationState::Validated {
            for correction in &profile.corrections {
                let pieces: Vec<&str> = correction.parameter_ref.split('.').collect();
                if pieces.len() == 3 {
                    if let Some(declared) =
                        declared_field(Some(calibration), pieces[0], pieces[1], pieces[2])
                    {
                        if let Some(previous) = declared.as_f64() {
                            if (previous - correction.value).abs() > 1e-12 {
                                finding(
                                    &mut findings,
                                    Severity::Information,
                                    "candidate_differs_from_declaration_not_drift",
                                    "profile",
                                    &correction.parameter_ref,
                                    "Historical proposed correction differs from an RGD declaration; without comparable observations and thresholds this is NOT evidence of physical drift.",
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    let preview = build_preview(profile, &binding, &joint_ids, &mut findings);
    findings.sort_by(|a, b| {
        (&a.source, &a.reference, &a.code).cmp(&(&b.source, &b.reference, &b.code))
    });
    Ok(PassiveReport {
        report_version: PASSIVE_REPORT_VERSION.to_owned(),
        model_id: model_id.to_owned(),
        source_digests: source_paths(bundle),
        profile_id: profile.map(|p| p.profile_id.clone()),
        profile_state: profile.map(|p| p.state),
        profile_binding: binding,
        claims_physical_calibration_certified: false,
        current_robot_position_observed: false,
        physical_drift_evaluated: false,
        coverage: counts,
        parameters,
        findings,
        export_preview: preview,
    })
}
