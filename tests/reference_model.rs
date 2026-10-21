//! A pinned Canonical Core example, not a physical robot or hardware reading.
use rgd_calibration::passive::{inspect_passive, BindingStatus, InspectionContext};
use rgd_calibration::source::BundleSource;
use std::path::PathBuf;

#[test]
fn pinned_openrgd_reference_has_expected_coverage_and_mismatched_ids() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/openrgd-reference");
    let bundle = BundleSource::load(&root).unwrap();
    let report = inspect_passive(&bundle, None, &InspectionContext::default()).unwrap();
    assert_eq!(report.model_id, "bhl_humanoid_lite_v2");
    assert_eq!(report.source_digests.len(), 5);
    assert_eq!(report.coverage.candidate_fields, 13);
    assert_eq!(report.coverage.declared_fields, 8);
    assert_eq!(report.coverage.saved_evidence_fields, 0);
    assert_eq!(report.coverage.without_saved_measurements, 13);
    assert_eq!(report.profile_binding, BindingStatus::NoProfile);
    assert!(report.export_preview.is_none());
    assert!(!report.current_robot_position_observed);
    assert!(!report.claims_physical_calibration_certified);
    assert!(!report.physical_drift_evaluated);

    assert!(report.findings.iter().any(|f| {
        f.code == "orphan_component_reference" && f.reference.contains("hip_pitch_right")
    }));
    assert!(report.findings.iter().any(|f| {
        f.code == "orphan_component_reference" && f.reference.contains("right_wrist_ft")
    }));
    assert!(report.findings.iter().any(|f| {
        f.code == "unresolved_cross_domain_reference" && f.reference.contains("knee_pitch_right")
    }));
    assert!(report.findings.iter().any(|f| {
        f.code == "unresolved_cross_domain_reference" && f.reference.contains("wifi_rf_monitor")
    }));
}
