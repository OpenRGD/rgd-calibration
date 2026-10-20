use rgd_calibration::{
    check_bindings, seal, transition, verify, CalibrationMode, CalibrationProfile,
    CalibrationState, Correction, CorrectionOperation, Evidence, EvidenceOrigin, FileStore,
    HardwareIdentity, JointZeroReference, Measurement, MeasurementKind, PositionObservation,
    PositionReferenceKind, ValidationRecord, fork_for_recalibration,
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
        observed_at_monotonic_ns: Some(1_000_000),
        reference_kind: PositionReferenceKind::AbsoluteEncoder,
        evidence_ids: vec!["sensor-frame-1".into()],
    };
    assert!(observation.has_fresh_absolute_reference_claim("boot-1", 1_005_000, 10_000));
    assert!(!observation.has_fresh_absolute_reference_claim("boot-2", 1_005_000, 10_000));
    assert!(!observation.has_fresh_absolute_reference_claim("boot-1", 1_100_000, 10_000));
    assert!(!observation.has_fresh_absolute_reference_claim("boot-1", 999_999, 10_000));
    observation.observed_at_monotonic_ns = None;
    assert!(!observation.has_fresh_absolute_reference_claim("boot-1", 1_005_000, 10_000));
    observation.observed_at_monotonic_ns = Some(1_000_000);
    observation.reference_kind = PositionReferenceKind::RelativeOnly;
    assert!(!observation.has_fresh_absolute_reference_claim("boot-1", 1_005_000, 10_000));
    observation.reference_kind = PositionReferenceKind::Unknown;
    assert!(!observation.has_fresh_absolute_reference_claim("boot-1", 1_005_000, 10_000));
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

#[test]
fn revision_fork_clears_old_evidence_and_preserves_immutable_parent() {
    let temp = tempfile::tempdir().unwrap();
    let store = FileStore::new(temp.path());
    let mut parent = sample_validated();
    seal(&mut parent).unwrap();
    store.save_new(&parent).unwrap();
    let mut child =
        fork_for_recalibration(&parent, "cal_0002", "2026-10-19T19:05:00Z").unwrap();
    assert_eq!(child.state, CalibrationState::CalibrationRequired);
    assert_eq!(child.previous_profile_id.as_deref(), Some("cal_0001"));
    assert_eq!(child.hardware, parent.hardware);
    assert!(child.measurements.is_empty());
    assert!(child.evidence.is_empty());
    assert!(child.corrections.is_empty());
    assert!(child.joint_zero_references.is_empty());
    assert!(child.validation.is_none());
    seal(&mut child).unwrap();
    store.save_new(&child).unwrap();
    assert_eq!(store.load("cal_0001").unwrap(), parent);
    assert_eq!(store.load("cal_0002").unwrap(), child);
}

#[test]
fn revision_rejects_reused_id_or_out_of_order_time() {
    let mut p = sample_validated();
    seal(&mut p).unwrap();
    assert!(fork_for_recalibration(&p, "cal_0001", "2026-10-19T19:06:00Z").is_err());
    assert!(fork_for_recalibration(&p, "cal_0002", "2026-10-19T18:00:00Z").is_err());
}

#[test]
fn orphan_revision_is_not_persisted() {
    let temp = tempfile::tempdir().unwrap();
    let store = FileStore::new(temp.path());
    let mut p = sample_validated();
    seal(&mut p).unwrap();
    let mut child =
        fork_for_recalibration(&p, "cal_0002", "2026-10-19T19:06:00Z").unwrap();
    seal(&mut child).unwrap();
    assert!(store.save_new(&child).is_err());
}

#[test]
fn invalid_duplicate_correction_is_rejected() {
    let mut p = sample_validated();
    p.corrections.push(p.corrections[0].clone());
    assert!(seal(&mut p).is_err());
}

#[test]
fn evidence_added_after_validation_invalidates_record() {
    let mut p = sample_validated();
    p.evidence[0].recorded_at = "2026-10-19T20:02:00Z".into();
    assert!(seal(&mut p).is_err());
}
