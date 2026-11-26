use rgd_calibration::passive::{inspect_passive, InspectionContext};
use rgd_calibration::source::{parse_source_object, BundleSource};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

const DESCRIPTION: &str = r#"{"meta_group":{"model_id_str":"cal021_synthetic"},"joint_topology_map":{},"sensors_mounting_map":{}}"#;

fn write(root: &Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn bundle(relative: &str, data: &[u8]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), relative, data);
    dir
}

#[test]
fn shared_syntax_corpus() {
    let corpus: Value = serde_json::from_str(include_str!(
        "fixtures/cal-021-a/bundle-syntax-corpus.v1.json"
    ))
    .unwrap();
    for case in corpus["cases"].as_array().unwrap() {
        let raw = case["raw"].as_str().unwrap().as_bytes();
        let jsonc = case["syntax"] == "jsonc";
        let expected = case["accepted"].as_bool().unwrap();
        assert_eq!(
            parse_source_object(raw, jsonc).is_ok(),
            expected,
            "syntax case {}",
            case["id"]
        );
    }
}

#[test]
fn invalid_utf8_is_rejected_even_inside_removed_comments() {
    assert!(parse_source_object(b"{/*\xff*/}", true).is_err());
}

#[test]
fn duplicate_keys_are_rejected_in_nested_arrays_and_strict_json() {
    assert!(parse_source_object(br#"{"x":[{"a":1,"a":2}]}"#, false).is_err());
}

#[test]
fn escaped_strings_remain_unchanged() {
    let data = br#"{"quote":"a\"b,}","slash":"x\\y","url":"https://example.test/a//b",}"#;
    let value = parse_source_object(data, true).unwrap();
    assert_eq!(value["quote"], "a\"b,}");
    assert_eq!(value["slash"], "x\\y");
    assert_eq!(value["url"], "https://example.test/a//b");
}

#[test]
fn source_digest_is_over_exact_original_bytes() {
    let raw = format!("// source metadata\n{DESCRIPTION}\n");
    let dir = bundle("spec/01_foundation/description.jsonc", raw.as_bytes());
    let loaded = BundleSource::load(dir.path()).unwrap();
    assert_eq!(
        loaded.description.raw_sha256,
        format!("sha256:{:x}", Sha256::digest(raw.as_bytes()))
    );
    assert_eq!(
        fs::read(dir.path().join("spec/01_foundation/description.jsonc")).unwrap(),
        raw.as_bytes()
    );
}

#[test]
fn spec_has_priority_and_no_cross_root_optional_backfill() {
    let dir = bundle(
        "spec/01_foundation/description.jsonc",
        DESCRIPTION.as_bytes(),
    );
    write(
        dir.path(),
        "standard/01_foundation/description.json",
        DESCRIPTION.as_bytes(),
    );
    write(dir.path(), "standard/01_foundation/calibration.json", b"{}");
    let loaded = BundleSource::load(dir.path()).unwrap();
    assert_eq!(
        loaded.description.path,
        "spec/01_foundation/description.jsonc"
    );
    assert!(loaded.calibration.is_none());
}

#[test]
fn malformed_preferred_spec_does_not_fall_back_to_valid_mirror() {
    let dir = bundle("spec/01_foundation/description.jsonc", b"{not_json: 1}");
    write(
        dir.path(),
        "standard/01_foundation/description.json",
        DESCRIPTION.as_bytes(),
    );
    let err = BundleSource::load(dir.path()).unwrap_err().to_string();
    assert!(err.contains("CAL_BUNDLE_SYNTAX"));
    assert!(err.contains("spec/01_foundation/description.jsonc"));
}

#[test]
fn same_role_dual_extensions_are_rejected() {
    let dir = bundle(
        "spec/01_foundation/description.jsonc",
        DESCRIPTION.as_bytes(),
    );
    write(
        dir.path(),
        "spec/01_foundation/description.json",
        DESCRIPTION.as_bytes(),
    );
    assert!(BundleSource::load(dir.path())
        .unwrap_err()
        .to_string()
        .contains("CAL_BUNDLE_AMBIGUOUS_SOURCE"));
}

#[test]
fn optional_role_dual_extensions_are_rejected() {
    let dir = bundle(
        "spec/01_foundation/description.jsonc",
        DESCRIPTION.as_bytes(),
    );
    write(dir.path(), "spec/01_foundation/calibration.jsonc", b"{}");
    write(dir.path(), "spec/01_foundation/calibration.json", b"{}");
    assert!(BundleSource::load(dir.path())
        .unwrap_err()
        .to_string()
        .contains("CAL_BUNDLE_AMBIGUOUS_SOURCE"));
}

#[test]
fn local_layouts_and_mixed_unique_roles_remain_supported() {
    for relative in [
        "spec/01_foundation/description.jsonc",
        "standard/01_foundation/description.json",
        "01_foundation/description.jsonc",
        "description.json",
    ] {
        let dir = bundle(relative, DESCRIPTION.as_bytes());
        let parent = Path::new(relative).parent().unwrap();
        write(
            dir.path(),
            parent.join("calibration.json").to_str().unwrap(),
            b"{}",
        );
        let loaded = BundleSource::load(dir.path()).unwrap();
        assert_eq!(loaded.description.path, relative);
        assert!(loaded.calibration.is_some());
    }
}

#[test]
fn absent_optional_sources_are_reported_not_invented() {
    let dir = bundle(
        "spec/01_foundation/description.jsonc",
        DESCRIPTION.as_bytes(),
    );
    let loaded = BundleSource::load(dir.path()).unwrap();
    let report = inspect_passive(&loaded, None, &InspectionContext::default()).unwrap();
    assert_eq!(report.coverage.candidate_fields, 0);
    assert_eq!(report.source_digests.len(), 1);
    assert_eq!(
        report
            .findings
            .iter()
            .filter(|f| f.code == "optional_source_not_provided")
            .count(),
        4
    );
    assert!(!report.current_robot_position_observed);
    assert!(!report.claims_physical_calibration_certified);
}

#[test]
fn incomplete_description_does_not_invent_model_id_or_invent_maps() {
    for value in [
        json!({"meta_group":{"model_id_str":" "},"joint_topology_map":{},"sensors_mounting_map":{}}),
        json!({"meta_group":{"model_id_str":"demo"},"joint_topology_map":{}}),
        json!({"meta_group":{"model_id_str":"demo"},"sensors_mounting_map":{}}),
        json!({"meta_group":{"model_id_str":"demo"},"joint_topology_map":{"j":3},"sensors_mounting_map":{}}),
    ] {
        let bytes = serde_json::to_vec(&value).unwrap();
        let dir = bundle("spec/01_foundation/description.jsonc", &bytes);
        let loaded = BundleSource::load(dir.path()).unwrap();
        let error = inspect_passive(&loaded, None, &InspectionContext::default()).unwrap_err();
        assert!(error.to_string().contains("CAL_BUNDLE_REQUIRED_FIELD"));
    }
}

#[test]
fn source_size_limit_is_checked_before_parsing() {
    let bytes = vec![b' '; 4 * 1024 * 1024 + 1];
    let dir = bundle("spec/01_foundation/description.jsonc", &bytes);
    assert!(BundleSource::load(dir.path())
        .unwrap_err()
        .to_string()
        .contains("CAL_BUNDLE_SIZE"));
}

#[cfg(unix)]
#[test]
fn source_symlink_and_symlinked_foundation_directory_are_rejected() {
    use std::os::unix::fs::symlink;
    let dir = bundle("actual/description.jsonc", DESCRIPTION.as_bytes());
    fs::create_dir_all(dir.path().join("spec")).unwrap();
    symlink(
        dir.path().join("actual"),
        dir.path().join("spec/01_foundation"),
    )
    .unwrap();
    assert!(BundleSource::load(dir.path())
        .unwrap_err()
        .to_string()
        .contains("CAL_BUNDLE_PATH"));
    fs::remove_file(dir.path().join("spec/01_foundation")).unwrap();
    fs::create_dir_all(dir.path().join("spec/01_foundation")).unwrap();
    symlink(
        dir.path().join("actual/description.jsonc"),
        dir.path().join("spec/01_foundation/description.jsonc"),
    )
    .unwrap();
    assert!(BundleSource::load(dir.path())
        .unwrap_err()
        .to_string()
        .contains("CAL_BUNDLE_PATH"));
}
