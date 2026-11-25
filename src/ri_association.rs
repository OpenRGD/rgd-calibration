//! CAL-RI-01: read-only association of RI v0.4 observation references with
//! independent Calibration artifacts. This is a *candidate interoperability
//! adapter*, not a canonical history, metrology, hardware or actuation service.
//!
//! Source contract pinned for review:
//! rgd-reality-ingress@5372f758df79c93fc887f3858d878635447f4a4d
//! contracts/consumer-observation-ref.v0.1-candidate.schema.json

use crate::model::{CalibrationProfile, CalibrationState};
use crate::validation::{verify, CalibrationError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const CAL_RI_SCHEMA_VERSION: &str = "cal.ri-observation-association.v0.1-candidate";
const RI_SCHEMA_VERSION: &str = "ri.consumer-observation-ref.v0.1-candidate";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    /// OCCHI PR #4 experiment: not a CAL-0.1 validated profile.
    ExperimentalEstimate,
    /// An independently sealed CAL profile. A SHA seal is not certification.
    SealedProfile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractMaturity {
    Experimental,
    Candidate,
}

/// Caller-supplied description of an existing CAL artifact. No writes, reads,
/// device access or identity authentication occur inside this adapter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationArtifactRef {
    pub artifact_kind: ArtifactKind,
    pub artifact_id: String,
    pub revision: String,
    /// Full SHA-256 of the artifact, NOT the RI observation payload hash.
    pub artifact_sha256: String,
    pub contract_maturity: ContractMaturity,
    pub expected_source_id: String,
    pub expected_modality: String,
    pub expected_reference_frame_id: String,
    pub expected_payload_media_type: String,
    /// Physical instance and nominal model are DIFFERENT things.
    pub physical_instance_id: Option<String>,
    pub model_ref: Option<String>,
    pub model_sha256: Option<String>,
    pub hardware_uid_claim: Option<String>,
    /// Transport metadata are unverified here: RI ref does not expose them.
    pub video_transport_format_claim: Option<String>,
    pub resolution_sbs_claim_wh: Option<[u32; 2]>,
    pub frame_a_id_claim: Option<String>,
    pub frame_b_id_claim: Option<String>,
    /// Bound to a capture session. Never infer 20 mm versus 25 mm.
    pub checkerboard_square_mm_claim: Option<f64>,
}

fn bad(message: &str) -> CalibrationError {
    CalibrationError::Invalid(message.to_owned())
}

fn is_hex64(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn require_str<'a>(
    object: &'a Value,
    key: &str,
    max_len: usize,
) -> Result<&'a str, CalibrationError> {
    let value = object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| bad(&format!("RI reference missing string: {key}")))?;
    if value.is_empty() || value.len() > max_len {
        return Err(bad(&format!("RI reference bad string length: {key}")));
    }
    Ok(value)
}

/// Fail closed on an unknown RI candidate schema or added authority fields.
/// This duplicates NO RI acquisition, hash calculation, time stamping or storage.
fn validate_ri_ref(input: &Value) -> Result<(), CalibrationError> {
    let source = input
        .as_object()
        .ok_or_else(|| bad("RI observation ref must be an object"))?;
    const KEYS: &[&str] = &[
        "schema_version",
        "observation_id",
        "source_id",
        "modality",
        "reference_frame_id",
        "sample_sequence",
        "payload_sha256_hex",
        "payload_media_type",
        "payload_size_bytes",
        "origin_status",
        "integrity_status",
        "source_verified",
        "robot_binding_verified",
        "history_committed",
        "actuation_enabled",
        "time",
    ];
    if source.len() != KEYS.len() || !source.keys().all(|key| KEYS.contains(&key.as_str())) {
        return Err(bad("unsupported RI reference shape or authority field"));
    }
    if require_str(input, "schema_version", 128)? != RI_SCHEMA_VERSION {
        return Err(bad("unsupported RI candidate contract"));
    }
    let id = require_str(input, "observation_id", 36)?;
    if !id.strip_prefix("obs_").is_some_and(|part| {
        part.len() == 32
            && part
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    }) {
        return Err(bad("invalid RI observation_id"));
    }
    require_str(input, "source_id", 256)?;
    require_str(input, "modality", 128)?;
    require_str(input, "reference_frame_id", 256)?;
    require_str(input, "payload_media_type", 128)?;
    if input
        .get("sample_sequence")
        .and_then(Value::as_u64)
        .is_none()
        || input
            .get("payload_size_bytes")
            .and_then(Value::as_u64)
            .is_none()
    {
        return Err(bad("invalid RI sample sequence or payload size"));
    }
    if !is_hex64(require_str(input, "payload_sha256_hex", 64)?) {
        return Err(bad("invalid RI payload digest"));
    }
    if ![
        "unknown",
        "source_claimed_physical",
        "source_declared_simulated",
    ]
    .contains(&require_str(input, "origin_status", 64)?)
    {
        return Err(bad("invalid RI origin status"));
    }
    if !["not_verified", "verified_local_hash"].contains(&require_str(
        input,
        "integrity_status",
        64,
    )?) {
        return Err(bad("invalid RI integrity status"));
    }
    for flag in [
        "source_verified",
        "robot_binding_verified",
        "history_committed",
        "actuation_enabled",
    ] {
        if input.get(flag).and_then(Value::as_bool) != Some(false) {
            return Err(bad(
                "RI candidate attempted to claim unauthorized authority",
            ));
        }
    }
    let time = input
        .get("time")
        .and_then(Value::as_object)
        .ok_or_else(|| bad("missing RI time envelope"))?;
    const TIME_KEYS: &[&str] = &[
        "clock_domain_id",
        "boot_id",
        "received_monotonic_ns",
        "source_time_status",
        "source_measurement_time",
        "chronograf_anchor_ref",
    ];
    if time.len() != TIME_KEYS.len() || !time.keys().all(|key| TIME_KEYS.contains(&key.as_str())) {
        return Err(bad("unsupported RI time fields"));
    }
    require_str(input.get("time").unwrap(), "clock_domain_id", 256)?;
    if !time
        .get("boot_id")
        .is_some_and(|v| v.is_null() || v.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 128))
        || time
            .get("received_monotonic_ns")
            .and_then(Value::as_u64)
            .is_none()
    {
        return Err(bad("invalid RI clock domain / boot / receipt time"));
    }
    if require_str(input.get("time").unwrap(), "source_time_status", 64)? != "unknown"
        || !time
            .get("source_measurement_time")
            .is_some_and(Value::is_null)
        || !time
            .get("chronograf_anchor_ref")
            .is_some_and(Value::is_null)
    {
        return Err(bad(
            "source timestamp or Chronograf authority cannot be asserted",
        ));
    }
    Ok(())
}

fn nonempty(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 256
}

fn validate_target(
    target: &CalibrationArtifactRef,
    profile: Option<&CalibrationProfile>,
) -> Result<Option<CalibrationState>, CalibrationError> {
    for s in [
        &target.artifact_id,
        &target.revision,
        &target.expected_source_id,
        &target.expected_modality,
        &target.expected_reference_frame_id,
        &target.expected_payload_media_type,
    ] {
        if !nonempty(s) {
            return Err(bad("missing or oversized CAL artifact reference field"));
        }
    }
    if !target
        .artifact_sha256
        .strip_prefix("sha256:")
        .is_some_and(is_hex64)
    {
        return Err(bad("invalid CAL artifact_sha256"));
    }
    for optional in [
        target.physical_instance_id.as_ref(),
        target.model_ref.as_ref(),
        target.model_sha256.as_ref(),
        target.hardware_uid_claim.as_ref(),
        target.video_transport_format_claim.as_ref(),
        target.frame_a_id_claim.as_ref(),
        target.frame_b_id_claim.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        if !nonempty(optional) {
            return Err(bad("invalid CAL artifact optional field"));
        }
    }
    if target
        .model_sha256
        .as_ref()
        .is_some_and(|x| !x.strip_prefix("sha256:").is_some_and(is_hex64))
    {
        return Err(bad("invalid CAL model_sha256 claim"));
    }
    if target
        .resolution_sbs_claim_wh
        .is_some_and(|[w, h]| w == 0 || h == 0)
    {
        return Err(bad("resolution must be nonzero"));
    }
    if target
        .checkerboard_square_mm_claim
        .is_some_and(|mm| !mm.is_finite() || mm <= 0.0)
    {
        return Err(bad("invalid checkerboard square size"));
    }
    match target.artifact_kind {
        ArtifactKind::ExperimentalEstimate => {
            if profile.is_some() || target.contract_maturity != ContractMaturity::Experimental {
                return Err(bad(
                    "experimental estimate cannot masquerade as a sealed CAL profile",
                ));
            }
            Ok(None)
        }
        ArtifactKind::SealedProfile => {
            if target.contract_maturity != ContractMaturity::Candidate {
                return Err(bad(
                    "sealed CAL profile must retain candidate maturity here",
                ));
            }
            let p = profile
                .ok_or_else(|| bad("sealed profile reference requires actual CAL record"))?;
            verify(p)?;
            if p.profile_id != target.artifact_id
                || p.profile_id != target.revision
                || p.integrity_sha256.as_deref() != Some(target.artifact_sha256.as_str())
                || target.physical_instance_id.as_deref() != Some(p.physical_instance_id.as_str())
                || target.model_ref.as_deref() != Some(p.model_ref.as_str())
                || target.model_sha256.as_deref() != Some(p.model_sha256.as_str())
            {
                return Err(bad(
                    "CAL profile revision / instance / model / digest mismatch",
                ));
            }
            Ok(Some(p.state))
        }
    }
}

/// Link an existing *read-only* RI consumer reference to an existing CAL
/// artifact. Returns only a new, separate sidecar: caller owns any persistence.
/// No hardware calls, mutation of history, timestamps, media reads, or depth.
pub fn associate_ri_observation(
    ri: &Value,
    candidate: Option<&CalibrationArtifactRef>,
    sealed_profile: Option<&CalibrationProfile>,
) -> Result<Value, CalibrationError> {
    validate_ri_ref(ri)?;
    if candidate.is_none() && sealed_profile.is_some() {
        return Err(bad("CAL profile supplied without calibration reference"));
    }
    let state = candidate
        .map(|target| validate_target(target, sealed_profile))
        .transpose()?
        .flatten();

    let mut reasons = Vec::<String>::new();
    let (association_status, calibration_ref) = if let Some(target) = candidate {
        let mut matches = true;
        for (field, expected, code) in [
            (
                "source_id",
                &target.expected_source_id,
                "source_id_mismatch",
            ),
            ("modality", &target.expected_modality, "modality_mismatch"),
            (
                "reference_frame_id",
                &target.expected_reference_frame_id,
                "reference_frame_mismatch",
            ),
            (
                "payload_media_type",
                &target.expected_payload_media_type,
                "payload_media_type_mismatch",
            ),
        ] {
            if ri.get(field).and_then(Value::as_str) != Some(expected.as_str()) {
                reasons.push(code.to_owned());
                matches = false;
            }
        }
        let status = if matches {
            "candidate_metadata_match"
        } else {
            "incompatible_metadata"
        };
        (status, serde_json::to_value(target)?)
    } else {
        reasons.push("calibration_reference_missing".into());
        ("missing_calibration", Value::Null)
    };
    if ri.get("integrity_status").and_then(Value::as_str) != Some("verified_local_hash") {
        reasons.push("ri_local_payload_hash_not_verified".into());
    }
    if ri.pointer("/time/boot_id").is_none_or(Value::is_null) {
        reasons.push("ri_boot_unknown".into());
    }
    if ri.get("origin_status").and_then(Value::as_str) != Some("source_claimed_physical") {
        reasons.push("source_physical_origin_not_established".into());
    }
    if state.is_some_and(|status| status != CalibrationState::Validated) {
        reasons.push("calibration_profile_not_validated".into());
    }
    if state.is_none() {
        reasons.push("independent_cal_profile_missing".into());
    }
    reasons.extend([
        "source_and_hardware_identity_unverified".into(),
        "video_dimensions_and_ab_order_unverified".into(),
        "stereo_metric_depth_not_independently_validated".into(),
        "camera_to_body_extrinsics_unverified".into(),
        "source_exposure_time_not_qualified".into(),
    ]);

    Ok(json!({
        "schema_version": CAL_RI_SCHEMA_VERSION,
        "observation_ref": ri.clone(),
        "calibration_ref": calibration_ref,
        "calibration_profile_state": state,
        "association_status": association_status,
        "reasons": reasons,
        "assurance": {
            "ri_reported_local_hash_verified": ri.get("integrity_status").and_then(Value::as_str) == Some("verified_local_hash"),
            "cal_profile_seal_verified": matches!(candidate.map(|c| c.artifact_kind), Some(ArtifactKind::SealedProfile)),
            "source_verified": false,
            "hardware_identity_verified": false,
            "robot_binding_verified": false,
            "metrology_certified": false,
            "history_committed": false,
            "actuation_enabled": false,
        },
        "depth_consumer": {
            "calibration_revision": candidate.map(|x| x.revision.clone()),
            "metric_depth_allowed": false,
            "output_unit": "unknown",
            "ab_physical_order_status": "unknown",
            "camera_to_body_extrinsics_status": "unknown",
            "source_exposure_time_status": "unknown",
            "status": "blocked_candidate",
        },
    }))
}
