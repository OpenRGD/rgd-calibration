//! D.4 synthetic historical profile generator.
//! Compiled ONLY in a disposable copy of the installed legacy source.
//! These are deliberately synthetic records, not real sensor measurements.
use rgd_calibration::{
    seal, verify, CalibrationProfile, Evidence, EvidenceOrigin, JointZeroReference,
    Measurement, MeasurementKind,
};
use serde_json::json;

fn profile(id: &str) -> CalibrationProfile {
    let digest = format!("sha256:{}", "a".repeat(64));
    let mut p = CalibrationProfile::new(
        id,
        "synthetic_d4_instance",
        "01_foundation/description.jsonc",
        digest.clone(),
        "2026-11-28T06:00:00Z",
    );
    p.evidence.push(Evidence {
        id: "simulated_fixture".into(),
        origin: EvidenceOrigin::Simulation,
        recorded_at: "2026-11-28T06:01:00Z".into(),
        source_ref: "synthetic://d4-test-only".into(),
        content_sha256: digest,
        chronon_ref: None,
    });
    p.measurements.push(Measurement {
        parameter_ref: "kinematic_offsets_map.right_knee.zero_point_offset_rad_float".into(),
        kind: MeasurementKind::Estimated,
        value: 0.125,
        unit: "rad".into(),
        standard_uncertainty: 0.0625,
        evidence_ids: vec!["simulated_fixture".into()],
    });
    p
}

fn main() {
    let mut good = profile("d4_legacy_good");
    let mut duplicate = profile("d4_legacy_duplicate");
    duplicate.measurements[0]
        .evidence_ids
        .push("simulated_fixture".into());
    let mut dangling = profile("d4_legacy_dangling");
    dangling.joint_zero_references.push(JointZeroReference {
        joint_ref: "right_knee".into(),
        encoder_zero_rad: None,
        mechanical_zero_rad: None,
        kinematic_zero_rad: None,
        initial_reference_rad: None,
        safe_rest_rad: None,
        homing_procedure_ref: None,
        evidence_ids: vec!["missing_synthetic_evidence".into()],
    });
    for p in [&mut good, &mut duplicate, &mut dangling] {
        seal(p).unwrap();
        verify(p).unwrap();
    }
    println!(
        "{}",
        json!({"good": good, "duplicate": duplicate, "dangling": dangling})
    );
}
