use rgd_calibration::{
    check_bindings, seal, transition, verify, CalibrationMode, CalibrationProfile,
    CalibrationState, Correction, CorrectionOperation, Evidence, EvidenceOrigin, FileStore,
    HardwareIdentity, JointZeroReference, Measurement, MeasurementKind, PositionObservation,
    PositionReferenceKind, ValidationRecord,
};

fn hash() -> String {
    format!("sha256:{}", "a".repeat(64))
}

fn sample_validated() -> CalibrationProfile {
    let mut p = CalibrationProfile::new(
        "cal_0001", "robot_unit_1", "01_foundation/description.jsonc",
        hash(), "2026-10-19T19:00:00Z",
    );
    p.mode = CalibrationMode::Guided;
    p.state = CalibrationState::InProgress;
    p.hardware.push(HardwareIdentity {
        component_ref: "joint_a_motor".into(),
        hardware_uid: Some("motor-123".into()),
        firmware_fingerprint: Some("fw-1".into()),
    });
    p.evidence.push(Evidence {
        id: "physical-01".into(),
        origin: EvidenceOrigin::HumanMeasurement,
        recorded_at: "2026-10-19T19:01:00Z".into(),
        source_ref: "local://session/001".into(),
        content_sha256: hash(),
        chronon_ref: None,
    });
    p.measurements.push(Measurement {
        parameter_ref: "joint_a.encoder_zero".into(),
        kind: MeasurementKind::Observed,
        value: 0.01,
        unit: "rad".into(),
        standard_uncertainty: 0.001,
        evidence_ids: vec!["physical-01".into()],
    });
    p.corrections.push(Correction {
        parameter_ref: "kinematic_offsets_map.joint_a.zero_point_offset_rad_float".into(),
        operation: CorrectionOperation::AdditiveDelta,
        value: 0.01,
        unit: "rad".into(),
        evidence_ids: vec!["physical-01".into()],
    });
    p.joint_zero_references.push(JointZeroReference {
        joint_ref: "joint_a".into(),
        encoder_zero_rad: Some(0.0),
        mechanical_zero_rad: None,
        kinematic_zero_rad: Some(0.01),
        initial_reference_rad: None,
        safe_rest_rad: None,
        homing_procedure_ref: None,
        evidence_ids: vec!["physical-01".into()],
    });
    p.validation = Some(ValidationRecord {
        passed: true,
        method: "independent human measurement".into(),
        validated_at: "2026-10-19T19:02:00Z".into(),
        approver: "operator-test".into(),
        evidence_ids: vec!["physical-01".into()],
    });
    transition(&mut p, CalibrationState::Validated).unwrap();
    p
}

#[test]
fn immutable_profile_roundtrip_and_no_overwrite() {
    let temp = tempfile::tempdir().unwrap();
    let store = FileStore::new(temp.path());
    let mut p = sample_validated();
    seal(&mut p).unwrap();
    let saved = store.save_new(&p).unwrap();
    assert!(saved.exists());
    assert_eq!(store.load("cal_0001").unwrap(), p);
    assert!(store.save_new(&p).is_err());
}

#[test]
fn tampering_fails_integrity_check() {
    let mut p = sample_validated();
    seal(&mut p).unwrap();
    p.corrections[0].value = 9.9;
    assert!(verify(&p).is_err());
}

#[test]
fn simulated_evidence_is_not_physical_validation() {
    let mut p = sample_validated();
    p.evidence[0].origin = EvidenceOrigin::Simulation;
    assert!(seal(&mut p).is_err());
}

#[test]
fn active_calibration_is_disabled_in_foundation() {
    let mut p = sample_validated();
    p.mode = CalibrationMode::Active;
    assert!(seal(&mut p).is_err());
}

#[test]
fn bindings_fail_closed_on_model_or_hardware_change() {
    let mut p = sample_validated();
    seal(&mut p).unwrap();
    let matching = p.hardware.clone();
    assert!(check_bindings(&p, "robot_unit_1", &hash(), &matching).is_ok());
    assert!(check_bindings(&p, "robot_unit_2", &hash(), &matching).is_err());
    let another_hash = format!("sha256:{}", "b".repeat(64));
    assert!(check_bindings(&p, "robot_unit_1", &another_hash, &matching).is_err());
    let mut changed = matching;
    changed[0].firmware_fingerprint = Some("fw-2".into());
    assert!(check_bindings(&p, "robot_unit_1", &hash(), &changed).is_err());
}

#[test]
fn unsafe_file_identifiers_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let store = FileStore::new(temp.path());
    assert!(store.load("../escape").is_err());
    assert!(store.load("/tmp/file").is_err());
}

#[test]
fn current_pose_cannot_be_inferred_from_old_boot_or_relative_encoder() {
    let mut observation = PositionObservation {
        joint_ref: "joint_a".into(),
        boot_id: "boot-1".into(),
        position_rad: Some(0.3),
        reference_kind: PositionReferenceKind::AbsoluteEncoder,
        evidence_ids: vec!["sensor-frame-1".into()],
    };
    assert!(observation.has_current_absolute_reference("boot-1"));
    assert!(!observation.has_current_absolute_reference("boot-2"));
    observation.reference_kind = PositionReferenceKind::RelativeOnly;
    assert!(!observation.has_current_absolute_reference("boot-1"));
    observation.reference_kind = PositionReferenceKind::Unknown;
    assert!(!observation.has_current_absolute_reference("boot-1"));
}

#[test]
fn transition_clears_validation_when_degraded() {
    let mut p = sample_validated();
    seal(&mut p).unwrap();
    transition(&mut p, CalibrationState::Degraded).unwrap();
    assert_eq!(p.state, CalibrationState::Degraded);
    assert!(p.validation.is_none());
    assert!(p.integrity_sha256.is_none());
    assert!(transition(&mut p, CalibrationState::Validated).is_err());
}

#[test]
fn missing_measurement_provenance_is_invalid() {
    let mut p = sample_validated();
    p.measurements[0].evidence_ids.clear();
    assert!(seal(&mut p).is_err());
}

#[test]
fn invalid_numeric_uncertainty_rejected() {
    let mut p = sample_validated();
    p.measurements[0].standard_uncertainty = f64::NAN;
    assert!(seal(&mut p).is_err());
}
