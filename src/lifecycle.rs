use crate::model::{CalibrationProfile, CalibrationState};
use crate::validation::{validate, CalibrationError};

fn allowed(from: CalibrationState, to: CalibrationState) -> bool {
    use CalibrationState::*;
    matches!(
        (from, to),
        (Uncalibrated, CalibrationRequired)
            | (CalibrationRequired, InProgress)
            | (InProgress, Validated)
            | (InProgress, CalibrationRequired)
            | (InProgress, Invalid)
            | (Validated, Degraded)
            | (Validated, Expired)
            | (Validated, Invalid)
            | (Validated, CalibrationRequired)
            | (Degraded, CalibrationRequired)
            | (Degraded, Invalid)
            | (Expired, CalibrationRequired)
            | (Expired, Invalid)
            | (Invalid, CalibrationRequired)
    )
}

/// Intentional state transition. Changed records must be resealed as new revisions.
/// This library cannot command motors or grant runtime motion permission.
pub fn transition(
    profile: &mut CalibrationProfile,
    next: CalibrationState,
) -> Result<(), CalibrationError> {
    if !allowed(profile.state, next) {
        return Err(CalibrationError::InvalidTransition);
    }
    let old_state = profile.state;
    let old_seal = profile.integrity_sha256.clone();
    let old_validation = profile.validation.clone();
    profile.state = next;
    if next != CalibrationState::Validated {
        profile.validation = None;
    }
    profile.integrity_sha256 = None;
    if let Err(error) = validate(profile) {
        profile.state = old_state;
        profile.integrity_sha256 = old_seal;
        profile.validation = old_validation;
        return Err(error);
    }
    Ok(())
}
