//! Synthetic fixtures only. No robot, motor, or physical measurements.
use rgd_calibration::model::{
    CalibrationMode, CalibrationProfile, CalibrationState, Correction, CorrectionOperation,
    Evidence, EvidenceOrigin, Measurement, MeasurementKind, ValidationRecord,
};
use rgd_calibration::passive::{inspect_passive, BindingStatus, CoverageKind, InspectionContext};
use rgd_calibration::source::{normalize_jsonc, BundleSource};
use rgd_calibration::validation::seal;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn sample_bundle() -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let foundation = dir.path().join("spec/01_foundation");
    fs::create_dir_all(&foundation).unwrap();
    let description = r#"
    {
      // Synthetic reference only, not a real robot.
      "meta_group": {"model_id_str": "synthetic_calibration_robot"},
      "joint_topology_map": {
        "right_knee": {"type_enum": "REVOLUTE",},
      },
      "sensors_mounting_map": {
        "main_cam": {
          "device_type_enum": "RGB_CAMERA",
          "mount_pose_xyz_m_vec3_float": [0.0, 0.0, 0.0],
          "mount_pose_rpy_rad_vec3_float": [0.0, 0.0, 0.0]
        },
        "torso_imu": {
          "device_type_enum": "IMU_6DOF",
          "mount_pose_xyz_m_vec3_float": [0.0, 0.0, 0.0],
          "mount_pose_rpy_rad_vec3_float": [0.0, 0.0, 0.0]
        },
      },
      "actuators_topology_map": {
        "knee_actuator": {"drives_joint_ref_str": "right_knee"}
      },
      "documentation_str": "https://example.org/path//suffix",
    }
    "#;
    fs::write(foundation.join("description.jsonc"), description).unwrap();
    fs::write(
        foundation.join("calibration.json"),
        serde_json::to_vec(&json!({
            "meta_group": {"file_role_enum": "CALIBRATION_DELTA_INSTANCE"},
            "kinematic_offsets_map": {
                "right_knee": {"zero_point_offset_rad_float": 0.02},
                "right_knee_old": {"zero_point_offset_rad_float": 0.03}
            },
            "sensor_extrinsics_delta_map": {
                "main_cam": {"mount_delta_xyz_m_vec3_float": [0.0, 0.0, 0.01]}
            },
            "sensor_intrinsics_map": {},
            "dynamics_correction_map": {"global_body": {"total_mass_delta_kg_float": 0.05}}
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        foundation.join("actuation_topology.json"),
        serde_json::to_vec(&json!({
            "joint_actuator_mapping_map": {
                "knee_pitch_right": {"target_joint_ref_str": "leg_right_knee_pitch"}
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        foundation.join("hal_mapping.json"),
        serde_json::to_vec(&json!({
            "actuator_drivers_map": {
                "knee_actuator": {"logical_actuator_ref_str": "knee_actuator"}
            },
            "sensor_drivers_map": {
                "wifi_rf_monitor": {"logical_sensor_ref_str": "wifi_rf_monitor"}
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        foundation.join("sensor_fidelity.json"),
        serde_json::to_vec(&json!({
            "sensor_error_profiles_map": {
                "main_cam": {}, "missing_ft": {}
            }
        }))
        .unwrap(),
    )
    .unwrap();
    dir
}

fn sha() -> String {
    format!("sha256:{}", "a".repeat(64))
}

fn approved_profile() -> CalibrationProfile {
    let mut p = CalibrationProfile::new(
        "synthetic_cal_01",
        "synthetic_unit",
        "01_foundation/description.jsonc",
        sha(),
        "2026-10-20T12:00:00Z",
    );
    p.mode = CalibrationMode::Guided;
    p.state = CalibrationState::Validated;
    p.evidence.push(Evidence {
        id: "claimed_human_01".into(),
        origin: EvidenceOrigin::HumanMeasurement,
        recorded_at: "2026-10-20T12:01:00Z".into(),
        source_ref: "synthetic://not-real".into(),
        content_sha256: sha(),
        chronon_ref: None,
    });
    p.measurements.push(Measurement {
        parameter_ref: "kinematic_offsets_map.right_knee.zero_point_offset_rad_float".into(),
        kind: MeasurementKind::Observed,
        value: 0.05,
        unit: "rad".into(),
        standard_uncertainty: 0.001,
        evidence_ids: vec!["claimed_human_01".into()],
    });
    p.corrections.push(Correction {
        parameter_ref: "kinematic_offsets_map.right_knee.zero_point_offset_rad_float".into(),
        operation: CorrectionOperation::AdditiveDelta,
        value: 0.05,
        unit: "rad".into(),
        evidence_ids: vec!["claimed_human_01".into()],
    });
    p.validation = Some(ValidationRecord {
        passed: true,
        method: "synthetic unit test fixture ONLY".into(),
        validated_at: "2026-10-20T12:02:00Z".into(),
        approver: "unit-test".into(),
        evidence_ids: vec!["claimed_human_01".into()],
    });
    seal(&mut p).unwrap();
    p
}

fn matching_context() -> InspectionContext {
    InspectionContext {
        physical_instance_id: Some("synthetic_unit".into()),
        model_sha256: Some(sha()),
    }
}

fn read_bundle(root: &Path) -> BundleSource {
    BundleSource::load(root).unwrap()
}

#[test]
fn jsonc_removes_comments_and_trailing_commas_without_touching_strings() {
    let sample = br#"{"url": "https://example.org/a//b", // comment
       "value": [1, 2,], /* other */ "quoted": "/* keep */",}"#;
    let value: Value = serde_json::from_slice(&normalize_jsonc(sample).unwrap()).unwrap();
    assert_eq!(value["url"], "https://example.org/a//b");
    assert_eq!(value["quoted"], "/* keep */");
    assert_eq!(value["value"], json!([1, 2]));
}

#[test]
fn unterminated_comment_is_rejected() {
    assert!(normalize_jsonc(br#"{"a":1, /* incomplete"#).is_err());
}

#[test]
fn inventory_is_passive_and_declared_is_not_measured() {
    let dir = sample_bundle();
    let bundle = read_bundle(dir.path());
    let report = inspect_passive(&bundle, None, &InspectionContext::default()).unwrap();
    assert_eq!(report.model_id, "synthetic_calibration_robot");
    assert_eq!(report.profile_binding, BindingStatus::NoProfile);
    assert_eq!(report.coverage.candidate_fields, 9);
    assert_eq!(report.coverage.declared_fields, 2);
    assert_eq!(report.coverage.saved_evidence_fields, 0);
    assert_eq!(report.coverage.without_saved_measurements, 9);
    assert!(!report.claims_physical_calibration_certified);
    assert!(!report.current_robot_position_observed);
    assert!(!report.physical_drift_evaluated);
    assert!(report.export_preview.is_none());
    assert!(report.parameters.iter().any(|p| p.parameter_ref
        == "kinematic_offsets_map.right_knee.zero_point_offset_rad_float"
        && p.coverage == CoverageKind::DeclaredOnly
        && p.stored_measurements.is_empty()));
}

#[test]
fn cross_domain_aliases_are_not_invented() {
    let dir = sample_bundle();
    let bundle = read_bundle(dir.path());
    let report = inspect_passive(&bundle, None, &InspectionContext::default()).unwrap();
    assert!(report.findings.iter().any(|f| {
        f.code == "unresolved_cross_domain_reference" && f.reference.contains("knee_pitch_right")
    }));
    assert!(report.findings.iter().any(|f| {
        f.code == "orphan_component_reference" && f.reference.contains("right_knee_old")
    }));
    assert!(report
        .findings
        .iter()
        .any(|f| { f.code == "orphan_component_reference" && f.reference.contains("missing_ft") }));
    assert!(report.findings.iter().any(|f| {
        f.code == "unresolved_cross_domain_reference" && f.reference.contains("wifi_rf_monitor")
    }));
}

#[test]
fn missing_optional_files_do_not_generate_fake_values() {
    let dir = sample_bundle();
    let foundation = dir.path().join("spec/01_foundation");
    for role in [
        "calibration",
        "actuation_topology",
        "hal_mapping",
        "sensor_fidelity",
    ] {
        fs::remove_file(foundation.join(format!("{role}.json"))).unwrap();
    }
    let bundle = read_bundle(dir.path());
    let report = inspect_passive(&bundle, None, &InspectionContext::default()).unwrap();
    assert_eq!(report.coverage.declared_fields, 0);
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "optional_source_not_provided"));
    assert!(report
        .source_digests
        .iter()
        .all(|s| s.role == "description"));
}

#[test]
fn saved_claim_requires_explicit_identity_before_preview() {
    let dir = sample_bundle();
    let bundle = read_bundle(dir.path());
    let profile = approved_profile();
    let report = inspect_passive(&bundle, Some(&profile), &InspectionContext::default()).unwrap();
    assert_eq!(report.profile_binding, BindingStatus::MissingContext);
    assert_eq!(report.coverage.saved_evidence_fields, 1);
    assert!(report.export_preview.is_none());
    let row = report
        .parameters
        .iter()
        .find(|x| x.parameter_ref == "kinematic_offsets_map.right_knee.zero_point_offset_rad_float")
        .unwrap();
    assert_eq!(row.stored_measurements[0].value, 0.05);
}

#[test]
fn matched_claim_only_produces_unapplied_delta_preview() {
    let dir = sample_bundle();
    let bundle = read_bundle(dir.path());
    let profile = approved_profile();
    let report = inspect_passive(&bundle, Some(&profile), &matching_context()).unwrap();
    assert_eq!(report.profile_binding, BindingStatus::ClaimedMatch);
    let preview = report.export_preview.unwrap();
    assert!(!preview.applied);
    assert!(!preview.hardware_binding_verified);
    assert_eq!(preview.status, "candidate_only_no_auto_commit");
    assert_eq!(preview.proposed_deltas.len(), 1);
    assert_eq!(
        preview.canonical_calibration_delta_patch["kinematic_offsets_map"]["right_knee"]
            ["zero_point_offset_rad_float"],
        json!(0.05)
    );
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "candidate_differs_from_declaration_not_drift"));
}

#[test]
fn identity_mismatch_rejects_candidate_preview() {
    let dir = sample_bundle();
    let bundle = read_bundle(dir.path());
    let profile = approved_profile();
    let mut context = matching_context();
    context.model_sha256 = Some(format!("sha256:{}", "b".repeat(64)));
    let report = inspect_passive(&bundle, Some(&profile), &context).unwrap();
    assert_eq!(report.profile_binding, BindingStatus::Mismatch);
    assert!(report.export_preview.is_none());
}

#[test]
fn unrecognized_path_cannot_be_exported() {
    let dir = sample_bundle();
    let bundle = read_bundle(dir.path());
    let mut profile = approved_profile();
    profile.corrections[0].parameter_ref =
        "kinematic_offsets_map.unknown_joint.zero_point_offset_rad_float".into();
    seal(&mut profile).unwrap();
    let report = inspect_passive(&bundle, Some(&profile), &matching_context()).unwrap();
    assert!(report.export_preview.is_none());
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "unsupported_preview_mapping"));
}

#[test]
fn unknown_operation_cannot_be_previewed() {
    let dir = sample_bundle();
    let bundle = read_bundle(dir.path());
    let mut profile = approved_profile();
    profile.corrections[0].operation = CorrectionOperation::AbsoluteOverride;
    seal(&mut profile).unwrap();
    let report = inspect_passive(&bundle, Some(&profile), &matching_context()).unwrap();
    assert!(report.export_preview.is_none());
}

#[test]
fn altered_seal_fails_before_inspection() {
    let dir = sample_bundle();
    let bundle = read_bundle(dir.path());
    let mut profile = approved_profile();
    profile.corrections[0].value = 0.5;
    assert!(inspect_passive(&bundle, Some(&profile), &matching_context()).is_err());
}

#[test]
fn invalid_vector_shape_is_not_silently_fixed() {
    let dir = sample_bundle();
    let calibration = dir.path().join("spec/01_foundation/calibration.json");
    let mut data: Value = serde_json::from_slice(&fs::read(&calibration).unwrap()).unwrap();
    data["sensor_extrinsics_delta_map"]["main_cam"]["mount_delta_xyz_m_vec3_float"] =
        json!([0.0, 0.2]);
    fs::write(&calibration, serde_json::to_vec(&data).unwrap()).unwrap();
    let bundle = read_bundle(dir.path());
    let report = inspect_passive(&bundle, None, &InspectionContext::default()).unwrap();
    assert_eq!(report.coverage.declared_fields, 1);
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "invalid_declared_shape"));
}

#[test]
fn unvalidated_profile_cannot_preview() {
    let dir = sample_bundle();
    let bundle = read_bundle(dir.path());
    let mut profile = approved_profile();
    profile.state = CalibrationState::Degraded;
    profile.validation = None;
    seal(&mut profile).unwrap();
    let report = inspect_passive(&bundle, Some(&profile), &matching_context()).unwrap();
    assert!(report.export_preview.is_none());
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "profile_not_validated"));
}
