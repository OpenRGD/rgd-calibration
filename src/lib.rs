//! OpenRGD Calibration — CAL-0.1 foundation and CAL-0.2 passive inspection.
//!
//! Offline-first, non-actuating calibration records.
//! No hardware drivers, command transport, or runtime motor authority.
//! A validated profile is not a permission to move.

pub mod lifecycle;
pub mod model;
pub mod passive;
pub mod revision;
pub mod source;
pub mod store;
pub mod validation;

pub use lifecycle::transition;
pub use model::{
    CalibrationMode, CalibrationProfile, CalibrationState, Correction, CorrectionOperation,
    Evidence, EvidenceOrigin, HardwareIdentity, JointZeroReference, Measurement, MeasurementKind,
    PositionObservation, PositionReferenceKind, ValidationRecord, PROFILE_SCHEMA_VERSION,
};
pub use revision::fork_for_recalibration;
pub use store::FileStore;
pub use validation::{check_bindings, seal, validate, verify, CalibrationError};
