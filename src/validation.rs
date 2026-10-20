use crate::model::{
    CalibrationMode, CalibrationProfile, CalibrationState, EvidenceOrigin, HardwareIdentity,
    PROFILE_SCHEMA_VERSION,
};
use chrono::DateTime;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Debug, thiserror::Error)]
pub enum CalibrationError {
    #[error("invalid calibration profile: {0}")]
    Invalid(String),
    #[error("profile integrity mismatch")]
    IntegrityMismatch,
    #[error("profile has no integrity seal")]
    Unsealed,
    #[error("profile binding mismatch: {0}")]
    BindingMismatch(String),
    #[error("invalid lifecycle transition")]
    InvalidTransition,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn validate_profile_id(id: &str) -> Result<(), CalibrationError> {
    if id.is_empty()
        || id.len() > 128
        || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(CalibrationError::Invalid("unsafe or empty profile_id".into()));
    }
    Ok(())
}

fn require(s: &str, what: &str) -> Result<(), CalibrationError> {
    if s.trim().is_empty() {
        return Err(CalibrationError::Invalid(format!("{what} must not be empty")));
    }
    Ok(())
}

fn sha256_digest_valid(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|digest| digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn check_time(value: &str, name: &str) -> Result<(), CalibrationError> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| CalibrationError::Invalid(format!("{name} requires RFC3339 timestamp")))?;
    Ok(())
}

fn check_evidence_refs(
    refs: &[String],
    ids: &HashSet<&str>,
    name: &str,
) -> Result<(), CalibrationError> {
    if refs.is_empty() || refs.iter().any(|id| !ids.contains(id.as_str())) {
        return Err(CalibrationError::Invalid(format!(
            "{name} must link to at least one existing evidence id"
        )));
    }
    Ok(())
}

/// Validate consistency of the record. This is NOT motor safety certification.
pub fn validate(profile: &CalibrationProfile) -> Result<(), CalibrationError> {
    if profile.schema_version != PROFILE_SCHEMA_VERSION {
        return Err(CalibrationError::Invalid("unsupported profile schema version".into()));
    }
    validate_profile_id(&profile.profile_id)?;
    if profile.previous_profile_id.as_deref() == Some(profile.profile_id.as_str()) {
        return Err(CalibrationError::Invalid("profile cannot supersede itself".into()));
    }
    if let Some(previous) = &profile.previous_profile_id {
        validate_profile_id(previous)?;
    }
    require(&profile.physical_instance_id, "physical_instance_id")?;
    require(&profile.model_ref, "model_ref")?;
    if !sha256_digest_valid(&profile.model_sha256) {
        return Err(CalibrationError::Invalid("model_sha256 must be sha256:hex64".into()));
    }
    check_time(&profile.created_at, "created_at")?;
    if profile.mode == CalibrationMode::Active {
        return Err(CalibrationError::Invalid(
            "active calibration is disabled in CAL-0.1".into(),
        ));
    }

    let mut evidence_ids = HashSet::new();
    for evidence in &profile.evidence {
        require(&evidence.id, "evidence.id")?;
        if !evidence_ids.insert(evidence.id.as_str()) {
            return Err(CalibrationError::Invalid("duplicate evidence id".into()));
        }
        require(&evidence.source_ref, "evidence.source_ref")?;
        check_time(&evidence.recorded_at, "evidence.recorded_at")?;
        if !sha256_digest_valid(&evidence.content_sha256) {
            return Err(CalibrationError::Invalid(
                "evidence.content_sha256 must be sha256:hex64".into(),
            ));
        }
        if let Some(link) = &evidence.chronon_ref {
            require(link, "evidence.chronon_ref")?;
        }
    }

    let mut device_ids = HashSet::new();
    for hardware in &profile.hardware {
        require(&hardware.component_ref, "hardware.component_ref")?;
        if !device_ids.insert(hardware.component_ref.as_str()) {
            return Err(CalibrationError::Invalid("duplicate hardware component_ref".into()));
        }
        if let Some(uid) = &hardware.hardware_uid {
            require(uid, "hardware.hardware_uid")?;
        }
        if let Some(firmware) = &hardware.firmware_fingerprint {
            require(firmware, "hardware.firmware_fingerprint")?;
        }
    }

    for measurement in &profile.measurements {
        require(&measurement.parameter_ref, "measurement.parameter_ref")?;
        require(&measurement.unit, "measurement.unit")?;
        if !measurement.value.is_finite()
            || !measurement.standard_uncertainty.is_finite()
            || measurement.standard_uncertainty < 0.0
        {
            return Err(CalibrationError::Invalid(
                "measurement value/uncertainty must be finite, uncertainty nonnegative".into(),
            ));
        }
        check_evidence_refs(&measurement.evidence_ids, &evidence_ids, "measurement")?;
    }
    for correction in &profile.corrections {
        require(&correction.parameter_ref, "correction.parameter_ref")?;
        require(&correction.unit, "correction.unit")?;
        if !correction.value.is_finite() {
            return Err(CalibrationError::Invalid("correction value must be finite".into()));
        }
        check_evidence_refs(&correction.evidence_ids, &evidence_ids, "correction")?;
    }
    let mut joint_ids = HashSet::new();
    for joint in &profile.joint_zero_references {
        require(&joint.joint_ref, "joint_zero_references.joint_ref")?;
        if !joint_ids.insert(joint.joint_ref.as_str()) {
            return Err(CalibrationError::Invalid("duplicate joint reference".into()));
        }
        let angles = [
            joint.encoder_zero_rad,
            joint.mechanical_zero_rad,
            joint.kinematic_zero_rad,
            joint.initial_reference_rad,
            joint.safe_rest_rad,
        ];
        if angles.iter().flatten().any(|value| !value.is_finite()) {
            return Err(CalibrationError::Invalid("joint references must be finite".into()));
        }
        if let Some(homing) = &joint.homing_procedure_ref {
            require(homing, "joint.homing_procedure_ref")?;
        }
        if angles.iter().any(Option::is_some) {
            check_evidence_refs(&joint.evidence_ids, &evidence_ids, "joint_zero_reference")?;
        }
    }
    if let Some(record) = &profile.validation {
        require(&record.method, "validation.method")?;
        require(&record.approver, "validation.approver")?;
        check_time(&record.validated_at, "validation.validated_at")?;
        check_evidence_refs(&record.evidence_ids, &evidence_ids, "validation")?;
    }

    if profile.state == CalibrationState::Validated {
        let record = profile.validation.as_ref().ok_or_else(|| {
            CalibrationError::Invalid("validated state requires a validation record".into())
        })?;
        if !record.passed {
            return Err(CalibrationError::Invalid(
                "validated state requires a passing result".into(),
            ));
        }
        let has_real_evidence = record.evidence_ids.iter().any(|id| {
            profile.evidence.iter().any(|e| {
                e.id == *id && e.origin.can_support_physical_validation()
            })
        });
        if !has_real_evidence {
            return Err(CalibrationError::Invalid(
                "simulated/derived/nominal evidence cannot alone validate a physical profile".into(),
            ));
        }
        for correction in &profile.corrections {
            let supported = correction.evidence_ids.iter().any(|id| {
                profile.evidence.iter().any(|e| {
                    e.id == *id && e.origin.can_support_physical_validation()
                })
            });
            if !supported {
                return Err(CalibrationError::Invalid(format!(
                    "correction {} lacks physical evidence",
                    correction.parameter_ref
                )));
            }
        }
    }
    Ok(())
}

fn calculated_digest(profile: &CalibrationProfile) -> Result<String, CalibrationError> {
    let mut unsealed = profile.clone();
    unsealed.integrity_sha256 = None;
    let json = serde_json::to_vec(&unsealed)?;
    let digest = Sha256::digest(&json);
    Ok(format!("sha256:{digest:x}"))
}

/// Integrity proves byte-level profile content identity, not metrological truth.
pub fn seal(profile: &mut CalibrationProfile) -> Result<(), CalibrationError> {
    validate(profile)?;
    profile.integrity_sha256 = Some(calculated_digest(profile)?);
    Ok(())
}

pub fn verify(profile: &CalibrationProfile) -> Result<(), CalibrationError> {
    validate(profile)?;
    match profile.integrity_sha256.as_deref() {
        None => Err(CalibrationError::Unsealed),
        Some(actual) if actual == calculated_digest(profile)? => Ok(()),
        Some(_) => Err(CalibrationError::IntegrityMismatch),
    }
}

/// Compare a sealed calibration against the currently discovered physical instance.
/// Missing/mismatched device identifiers fail closed when a profile binds them.
pub fn check_bindings(
    profile: &CalibrationProfile,
    physical_instance_id: &str,
    model_sha256: &str,
    discovered_hardware: &[HardwareIdentity],
) -> Result<(), CalibrationError> {
    verify(profile)?;
    if physical_instance_id != profile.physical_instance_id {
        return Err(CalibrationError::BindingMismatch("robot instance changed".into()));
    }
    if model_sha256 != profile.model_sha256 {
        return Err(CalibrationError::BindingMismatch("RGD model hash changed".into()));
    }
    for expected in &profile.hardware {
        let found = discovered_hardware
            .iter()
            .find(|device| device.component_ref == expected.component_ref)
            .ok_or_else(|| {
                CalibrationError::BindingMismatch(format!(
                    "device missing: {}",
                    expected.component_ref
                ))
            })?;
        if expected.hardware_uid != found.hardware_uid
            || expected.firmware_fingerprint != found.firmware_fingerprint
        {
            return Err(CalibrationError::BindingMismatch(format!(
                "device identity or firmware changed: {}",
                expected.component_ref
            )));
        }
    }
    Ok(())
}

/// Exposed for consumers creating source references without asserting provenance.
pub fn is_physical_source(origin: EvidenceOrigin) -> bool {
    origin.can_support_physical_validation()
}
