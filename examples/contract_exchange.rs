//! Test-only exchange with the pinned upstream RI producer.
//! All profiles and numeric observations in this example are synthetic fixtures.
//! Never invoke this example for a physical robot or publish its data as evidence.
use rgd_calibration::passive::{inspect_passive, InspectionContext};
use rgd_calibration::ri_association::{
    associate_ri_observation, ArtifactKind, CalibrationArtifactRef, ContractMaturity,
};
use rgd_calibration::source::BundleSource;
use rgd_calibration::{
    seal, validate, verify, CalibrationMode, CalibrationProfile, CalibrationState, Correction,
    CorrectionOperation, Evidence, EvidenceOrigin, HardwareIdentity, JointZeroReference,
    Measurement, MeasurementKind, PositionObservation, PositionReferenceKind, ValidationRecord,
};
use serde_json::{json, Map, Value};
use std::fs;
use std::io;

fn hash() -> String {
    format!("sha256:{}", "a".repeat(64))
}

fn profile() -> CalibrationProfile {
    let mut p = CalibrationProfile::new(
        "synthetic_contract_profile",
        "synthetic_instance",
        "01_foundation/description.jsonc",
        hash(),
        "2026-11-26T10:00:00Z",
    );
    p.mode = CalibrationMode::Guided;
    p.state = CalibrationState::Validated;
    p.hardware.push(HardwareIdentity {
        component_ref: "synthetic_motor".into(),
        hardware_uid: Some("synthetic_uid".into()),
        firmware_fingerprint: None,
    });
    p.evidence.push(Evidence {
        id: "synthetic_claim".into(),
        origin: EvidenceOrigin::HumanMeasurement,
        recorded_at: "2026-11-26T10:01:00Z".into(),
        source_ref: "synthetic://not-a-real-observation".into(),
        content_sha256: hash(),
        chronon_ref: None,
    });
    let param = "kinematic_offsets_map.right_knee.zero_point_offset_rad_float";
    p.measurements.push(Measurement {
        parameter_ref: param.into(),
        kind: MeasurementKind::Observed,
        value: 0.05,
        unit: "rad".into(),
        standard_uncertainty: 0.001,
        evidence_ids: vec!["synthetic_claim".into()],
    });
    p.corrections.push(Correction {
        parameter_ref: param.into(),
        operation: CorrectionOperation::AdditiveDelta,
        value: 0.05,
        unit: "rad".into(),
        evidence_ids: vec!["synthetic_claim".into()],
    });
    p.joint_zero_references.push(JointZeroReference {
        joint_ref: "right_knee".into(),
        encoder_zero_rad: None,
        mechanical_zero_rad: None,
        kinematic_zero_rad: None,
        initial_reference_rad: None,
        safe_rest_rad: None,
        homing_procedure_ref: None,
        evidence_ids: vec![],
    });
    p.validation = Some(ValidationRecord {
        passed: true,
        method: "SYNTHETIC TEST ONLY".into(),
        validated_at: "2026-11-26T10:02:00Z".into(),
        approver: "synthetic_test".into(),
        evidence_ids: vec!["synthetic_claim".into()],
    });
    seal(&mut p).unwrap();
    p
}

fn artifact() -> CalibrationArtifactRef {
    CalibrationArtifactRef {
        artifact_kind: ArtifactKind::ExperimentalEstimate,
        artifact_id: "synthetic_occhi_estimate".into(),
        revision: "synthetic_revision".into(),
        artifact_sha256: hash(),
        contract_maturity: ContractMaturity::Experimental,
        expected_source_id: "source.synthetic.camera".into(),
        expected_modality: "rgb".into(),
        expected_reference_frame_id: "frame.camera.synthetic".into(),
        expected_payload_media_type: "image/jpeg".into(),
        physical_instance_id: Some("synthetic_instance".into()),
        model_ref: Some("01_foundation/description.jsonc".into()),
        model_sha256: Some(hash()),
        hardware_uid_claim: None,
        video_transport_format_claim: None,
        resolution_sbs_claim_wh: None,
        frame_a_id_claim: None,
        frame_b_id_claim: None,
        checkerboard_square_mm_claim: Some(20.0),
    }
}

fn sealed_artifact(p: &CalibrationProfile) -> CalibrationArtifactRef {
    let mut a = artifact();
    a.artifact_kind = ArtifactKind::SealedProfile;
    a.contract_maturity = ContractMaturity::Candidate;
    a.artifact_id = p.profile_id.clone();
    a.revision = p.profile_id.clone();
    a.artifact_sha256 = p.integrity_sha256.clone().unwrap();
    a
}

fn main() {
    let refs: Map<String, Value> = serde_json::from_reader(io::stdin().lock()).unwrap();
    let p = profile();
    let mut expired = p.clone();
    expired.state = CalibrationState::Expired;
    seal(&mut expired).unwrap();
    let mut no_profile = CalibrationProfile::new(
        "synthetic_empty_profile",
        "synthetic_instance",
        "01_foundation/description.jsonc",
        hash(),
        "2026-11-26T10:00:00Z",
    );
    seal(&mut no_profile).unwrap();
    let folder = tempfile::tempdir().unwrap();
    let foundation = folder.path().join("spec/01_foundation");
    fs::create_dir_all(&foundation).unwrap();
    fs::write(
        foundation.join("description.jsonc"),
        br#"{
          "meta_group":{"model_id_str":"synthetic_contract_model"},
          "joint_topology_map":{"right_knee":{}},
          "sensors_mounting_map":{}
        }"#,
    )
    .unwrap();
    let bundle = BundleSource::load(folder.path()).unwrap();
    let context = InspectionContext {
        physical_instance_id: Some(p.physical_instance_id.clone()),
        model_sha256: Some(p.model_sha256.clone()),
    };
    let passive = inspect_passive(&bundle, None, &InspectionContext::default()).unwrap();
    let with_profile = inspect_passive(&bundle, Some(&p), &context).unwrap();
    assert!(with_profile.export_preview.is_some());
    let position = PositionObservation {
        joint_ref: "right_knee".into(),
        boot_id: "synthetic_boot".into(),
        position_rad: None,
        observed_at_monotonic_ns: None,
        reference_kind: PositionReferenceKind::Unknown,
        evidence_ids: vec![],
    };
    let mut associated = Map::new();
    for (name, ri) in refs {
        let row = match associate_ri_observation(&ri, Some(&artifact()), None) {
            Ok(experimental) => {
                let missing = associate_ri_observation(&ri, None, None).unwrap();
                let sealed =
                    associate_ri_observation(&ri, Some(&sealed_artifact(&p)), Some(&p)).unwrap();
                let stale =
                    associate_ri_observation(&ri, Some(&sealed_artifact(&expired)), Some(&expired))
                        .unwrap();
                json!({
                    "accepted":true,
                    "experimental":experimental,
                    "missing":missing,
                    "sealed":sealed,
                    "expired":stale
                })
            }
            Err(_) => json!({"accepted":false}),
        };
        associated.insert(name, row);
    }
    // Structural JSON acceptance alone cannot validate this profile.
    let mut active = p.clone();
    active.mode = CalibrationMode::Active;
    let mut dangling = p.clone();
    dangling.measurements[0].evidence_ids = vec!["missing".into()];
    let mut duplicate = p.clone();
    duplicate.measurements[0]
        .evidence_ids
        .push("synthetic_claim".into());
    let mut unknown_zero = p.clone();
    unknown_zero.joint_zero_references[0].evidence_ids = vec!["missing".into()];
    let mut altered = p.clone();
    altered.measurements[0].value = 9.0;
    println!(
        "{}",
        json!({
            "profile":p,
            "empty_profile":no_profile,
            "unknown_position":position,
            "passive":passive,
            "with_profile":with_profile,
            "ri":associated,
            "runtime_rejections":{
                "active_mode":validate(&active).is_err(),
                "dangling_evidence":validate(&dangling).is_err(),
                "duplicate_evidence_ref":validate(&duplicate).is_err(),
                "unknown_zero_dangling_evidence":validate(&unknown_zero).is_err(),
                "changed_sealed_profile":verify(&altered).is_err()
            }
        })
    );
}
