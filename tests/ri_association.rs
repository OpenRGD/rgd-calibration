use rgd_calibration::model::{CalibrationProfile, CalibrationState};
use rgd_calibration::ri_association::{
    associate_ri_observation, ArtifactKind, CalibrationArtifactRef, ContractMaturity,
    CAL_RI_SCHEMA_VERSION,
};
use rgd_calibration::seal;
use serde_json::{json, Value};

fn sha_hex(value: char) -> String {
    format!("sha256:{}", value.to_string().repeat(64))
}

fn reference() -> Value {
    serde_json::from_str(include_str!("fixtures/ri-consumer-ref.v0.1-candidate.json")).unwrap()
}

fn experimental() -> CalibrationArtifactRef {
    CalibrationArtifactRef {
        artifact_kind: ArtifactKind::ExperimentalEstimate,
        artifact_id: "occhi_stereo_session_01".into(),
        revision: "estimate_01".into(),
        artifact_sha256: sha_hex('a'),
        contract_maturity: ContractMaturity::Experimental,
        expected_source_id: "source.synthetic.occhi".into(),
        expected_modality: "rgb".into(),
        expected_reference_frame_id: "frame.synthetic.occhi_sbs".into(),
        expected_payload_media_type: "image/jpeg".into(),
        physical_instance_id: Some("paky01_claim_only".into()),
        model_ref: Some("01_foundation/description.jsonc".into()),
        model_sha256: Some(sha_hex('b')),
        hardware_uid_claim: Some("32e4:2b10:claimed".into()),
        video_transport_format_claim: Some("MJPG_SBS".into()),
        resolution_sbs_claim_wh: Some([2560, 720]),
        frame_a_id_claim: Some("camera_A_unconfirmed".into()),
        frame_b_id_claim: Some("camera_B_unconfirmed".into()),
        checkerboard_square_mm_claim: Some(20.0),
    }
}

fn sealed() -> CalibrationProfile {
    let mut profile = CalibrationProfile::new(
        "cal_session_0001",
        "paky01_claim_only",
        "01_foundation/description.jsonc",
        sha_hex('b'),
        "2026-11-25T12:00:00Z",
    );
    seal(&mut profile).unwrap();
    profile
}

fn from_sealed(profile: &CalibrationProfile) -> CalibrationArtifactRef {
    let mut t = experimental();
    t.artifact_kind = ArtifactKind::SealedProfile;
    t.artifact_id = profile.profile_id.clone();
    t.revision = profile.profile_id.clone();
    t.artifact_sha256 = profile.integrity_sha256.clone().unwrap();
    t.contract_maturity = ContractMaturity::Candidate;
    t
}

#[test]
fn experimental_reference_is_sidecar_and_never_updates_ri_ref() {
    let input = reference();
    let original = input.clone();
    let cal = experimental();
    let result = associate_ri_observation(&input, Some(&cal), None).unwrap();
    assert_eq!(input, original);
    assert_eq!(result["schema_version"], CAL_RI_SCHEMA_VERSION);
    assert_eq!(result["observation_ref"], input);
    assert_eq!(result["observation_ref"]["observation_id"], original["observation_id"]);
    assert_eq!(result["observation_ref"]["payload_sha256_hex"], original["payload_sha256_hex"]);
    assert_eq!(result["calibration_ref"]["revision"], "estimate_01");
    assert_eq!(result["calibration_profile_state"], Value::Null);
    assert_eq!(result["association_status"], "candidate_metadata_match");
    assert_eq!(result["assurance"]["ri_reported_local_hash_verified"], true);
    assert_eq!(result["assurance"]["cal_profile_seal_verified"], false);
    assert_eq!(result["assurance"]["source_verified"], false);
    assert_eq!(result["assurance"]["robot_binding_verified"], false);
    assert_eq!(result["assurance"]["actuation_enabled"], false);
    assert_eq!(result["depth_consumer"]["metric_depth_allowed"], false);
    assert_eq!(result["depth_consumer"]["output_unit"], "unknown");
    assert!(result.get("payload_path").is_none());
    assert!(result.get("payload_bytes").is_none());
}

#[test]
fn missing_calibration_is_explicit_and_never_depth_usable() {
    let result = associate_ri_observation(&reference(), None, None).unwrap();
    assert_eq!(result["association_status"], "missing_calibration");
    assert_eq!(result["calibration_ref"], Value::Null);
    assert_eq!(result["depth_consumer"]["metric_depth_allowed"], false);
}

#[test]
fn candidate_observation_without_local_hash_verification_is_flagged() {
    let mut ri = reference();
    ri["integrity_status"] = json!("not_verified");
    let result = associate_ri_observation(&ri, Some(&experimental()), None).unwrap();
    assert_eq!(result["assurance"]["ri_reported_local_hash_verified"], false);
    assert!(result["reasons"].as_array().unwrap().iter().any(|r| r == "ri_local_payload_hash_not_verified"));
}

#[test]
fn mismatched_source_frame_media_and_modality_are_never_associated_as_compatible() {
    for (field, value, expected) in [
        ("source_id", "different_owner", "source_id_mismatch"),
        ("reference_frame_id", "other.frame", "reference_frame_mismatch"),
        ("payload_media_type", "video/raw", "payload_media_type_mismatch"),
        ("modality", "audio", "modality_mismatch"),
    ] {
        let mut ri = reference();
        ri[field] = json!(value);
        let out = associate_ri_observation(&ri, Some(&experimental()), None).unwrap();
        assert_eq!(out["association_status"], "incompatible_metadata");
        assert!(out["reasons"].as_array().unwrap().iter().any(|v| v == expected));
        assert_eq!(out["depth_consumer"]["metric_depth_allowed"], false);
    }
}

#[test]
fn invalid_ri_id_hash_and_unknown_authority_fields_are_rejected() {
    for (key, bad_value) in [
        ("observation_id", json!("obs_123")),
        ("payload_sha256_hex", json!("z".repeat(64))),
        ("sample_sequence", json!(-1)),
        ("payload_size_bytes", json!(-5)),
        ("schema_version", json!("other")),
    ] {
        let mut ri = reference();
        ri[key] = bad_value;
        assert!(associate_ri_observation(&ri, None, None).is_err());
    }
    let mut ri = reference();
    ri["made_up_authority"] = json!("certified");
    assert!(associate_ri_observation(&ri, None, None).is_err());
}

#[test]
fn fabricated_ri_source_robot_history_motion_or_chronograf_authority_is_rejected() {
    for flag in [
        "source_verified",
        "robot_binding_verified",
        "history_committed",
        "actuation_enabled",
    ] {
        let mut ri = reference();
        ri[flag] = json!(true);
        assert!(associate_ri_observation(&ri, Some(&experimental()), None).is_err());
    }
    let mut ri = reference();
    ri["time"]["chronograf_anchor_ref"] = json!("fabricated-anchor");
    assert!(associate_ri_observation(&ri, None, None).is_err());
    ri = reference();
    ri["time"]["source_measurement_time"] = json!("2026-11-25T00:00:00Z");
    assert!(associate_ri_observation(&ri, None, None).is_err());
}

#[test]
fn missing_boot_and_unknown_capture_time_do_not_create_time_authority() {
    let mut ri = reference();
    ri["time"]["boot_id"] = Value::Null;
    let result = associate_ri_observation(&ri, Some(&experimental()), None).unwrap();
    assert!(result["reasons"].as_array().unwrap().iter().any(|v| v == "ri_boot_unknown"));
    assert_eq!(result["depth_consumer"]["source_exposure_time_status"], "unknown");
    assert_eq!(result["assurance"]["history_committed"], false);
}

#[test]
fn source_claimed_physical_still_does_not_verify_source_or_certify_depth() {
    let mut ri = reference();
    ri["origin_status"] = json!("source_claimed_physical");
    let out = associate_ri_observation(&ri, Some(&experimental()), None).unwrap();
    assert_eq!(out["assurance"]["source_verified"], false);
    assert_eq!(out["assurance"]["metrology_certified"], false);
    assert_eq!(out["depth_consumer"]["metric_depth_allowed"], false);
}

#[test]
fn no_automatic_upgrade_from_experiment_to_candidate_or_validated() {
    let mut target = experimental();
    target.contract_maturity = ContractMaturity::Candidate;
    assert!(associate_ri_observation(&reference(), Some(&target), None).is_err());
    target = experimental();
    let profile = sealed();
    assert!(associate_ri_observation(&reference(), Some(&target), Some(&profile)).is_err());
}

#[test]
fn malformed_candidate_digest_resolution_board_size_and_frame_are_rejected() {
    let mut target = experimental();
    target.artifact_sha256 = "sha256:bad".into();
    assert!(associate_ri_observation(&reference(), Some(&target), None).is_err());
    target = experimental();
    target.resolution_sbs_claim_wh = Some([0, 720]);
    assert!(associate_ri_observation(&reference(), Some(&target), None).is_err());
    target = experimental();
    target.checkerboard_square_mm_claim = Some(f64::NAN);
    assert!(associate_ri_observation(&reference(), Some(&target), None).is_err());
    target = experimental();
    target.checkerboard_square_mm_claim = Some(0.0);
    assert!(associate_ri_observation(&reference(), Some(&target), None).is_err());
    target = experimental();
    target.expected_reference_frame_id.clear();
    assert!(associate_ri_observation(&reference(), Some(&target), None).is_err());
}

#[test]
fn sealed_profile_reference_requires_real_record_revision_and_sha() {
    let p = sealed();
    let mut target = from_sealed(&p);
    assert!(associate_ri_observation(&reference(), Some(&target), None).is_err());
    target.revision = "fabricated_increment".into();
    assert!(associate_ri_observation(&reference(), Some(&target), Some(&p)).is_err());
    target = from_sealed(&p);
    target.physical_instance_id = Some("other_robot".into());
    assert!(associate_ri_observation(&reference(), Some(&target), Some(&p)).is_err());
    target = from_sealed(&p);
    target.model_sha256 = Some(sha_hex('c'));
    assert!(associate_ri_observation(&reference(), Some(&target), Some(&p)).is_err());
    target = from_sealed(&p);
    target.artifact_sha256 = sha_hex('c');
    assert!(associate_ri_observation(&reference(), Some(&target), Some(&p)).is_err());
}

#[test]
fn authentic_sealed_profile_is_not_a_physical_calibration_certificate() {
    let p = sealed();
    let target = from_sealed(&p);
    let out = associate_ri_observation(&reference(), Some(&target), Some(&p)).unwrap();
    assert_eq!(out["assurance"]["cal_profile_seal_verified"], true);
    assert_eq!(out["calibration_profile_state"], "uncalibrated");
    assert!(out["reasons"].as_array().unwrap().iter().any(|v| v == "calibration_profile_not_validated"));
    assert_eq!(out["assurance"]["metrology_certified"], false);
    assert_eq!(out["depth_consumer"]["metric_depth_allowed"], false);
}

#[test]
fn expired_sealed_profile_is_linkable_only_as_blocked_historical_context() {
    let mut p = sealed();
    p.state = CalibrationState::Expired;
    seal(&mut p).unwrap();
    let target = from_sealed(&p);
    let out = associate_ri_observation(&reference(), Some(&target), Some(&p)).unwrap();
    assert_eq!(out["calibration_profile_state"], "expired");
    assert_eq!(out["depth_consumer"]["metric_depth_allowed"], false);
}

#[test]
fn tampered_sealed_profile_is_rejected() {
    let mut p = sealed();
    let target = from_sealed(&p);
    p.model_ref.push_str(".tampered");
    assert!(associate_ri_observation(&reference(), Some(&target), Some(&p)).is_err());
}

#[test]
fn sidecar_json_roundtrip_preserves_ri_ids_hash_and_unknown_physical_truth() {
    let output = associate_ri_observation(&reference(), Some(&experimental()), None).unwrap();
    let bytes = serde_json::to_vec(&output).unwrap();
    let decoded: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded["observation_ref"], reference());
    assert_eq!(decoded["depth_consumer"]["calibration_revision"], "estimate_01");
    assert_eq!(decoded["depth_consumer"]["ab_physical_order_status"], "unknown");
    assert_eq!(decoded["depth_consumer"]["camera_to_body_extrinsics_status"], "unknown");
}
