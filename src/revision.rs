use crate::model::{CalibrationProfile, CalibrationState};
use crate::validation::{validate, validate_profile_id, verify, CalibrationError};
use chrono::DateTime;

/// Begin a new, unvalidated, physically instance-bound revision.
///
/// Deliberately discards measurements, corrections, and evidence. None of the
/// previous revision's measurements may be silently re-certified after drift,
/// maintenance, reboot, or a hardware change. The historical parent remains
/// immutable in its FileStore.
pub fn fork_for_recalibration(
    parent: &CalibrationProfile,
    new_profile_id: impl Into<String>,
    created_at: impl Into<String>,
) -> Result<CalibrationProfile, CalibrationError> {
    verify(parent)?;
    let new_profile_id = new_profile_id.into();
    let created_at = created_at.into();
    validate_profile_id(&new_profile_id)?;
    if new_profile_id == parent.profile_id {
        return Err(CalibrationError::Invalid(
            "new revision must have a different profile_id".into(),
        ));
    }
    let parent_time = DateTime::parse_from_rfc3339(&parent.created_at)
        .map_err(|_| CalibrationError::Invalid("invalid parent created_at".into()))?;
    let revision_time = DateTime::parse_from_rfc3339(&created_at)
        .map_err(|_| CalibrationError::Invalid("invalid revision created_at".into()))?;
    if revision_time <= parent_time {
        return Err(CalibrationError::Invalid(
            "revision created_at must postdate predecessor".into(),
        ));
    }

    let mut next = CalibrationProfile::new(
        new_profile_id,
        parent.physical_instance_id.clone(),
        parent.model_ref.clone(),
        parent.model_sha256.clone(),
        created_at,
    );
    next.state = CalibrationState::CalibrationRequired;
    next.hardware = parent.hardware.clone(); // Binding hints, not verified discoveries.
    next.previous_profile_id = Some(parent.profile_id.clone());
    validate(&next)?;
    Ok(next)
}
