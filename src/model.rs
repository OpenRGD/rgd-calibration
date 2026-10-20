use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PROFILE_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationMode {
    Passive,
    Guided,
    /// Reserved for a later safety-gated runtime. Rejected in v0.1 validation.
    Active,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationState {
    Uncalibrated,
    CalibrationRequired,
    InProgress,
    Validated,
    Degraded,
    Expired,
    Invalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceOrigin {
    PhysicalSensor,
    HumanMeasurement,
    Simulation,
    NominalModel,
    Derived,
}

impl EvidenceOrigin {
    pub fn can_support_physical_validation(self) -> bool {
        matches!(self, Self::PhysicalSensor | Self::HumanMeasurement)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementKind {
    Observed,
    Estimated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionOperation {
    AdditiveDelta,
    MultiplicativeFactor,
    AbsoluteOverride,
}

/// Reference to externally stored source material, never a fabricated reading.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub id: String,
    pub origin: EvidenceOrigin,
    pub recorded_at: String,
    pub source_ref: String,
    pub content_sha256: String,
    /// Optional link only. Calibration does not mint canonical Chronons.
    pub chronon_ref: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Measurement {
    /// Stable model-relative parameter path; exact mapping is adapter-owned.
    pub parameter_ref: String,
    pub kind: MeasurementKind,
    pub value: f64,
    pub unit: String,
    pub standard_uncertainty: f64,
    pub evidence_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correction {
    pub parameter_ref: String,
    pub operation: CorrectionOperation,
    pub value: f64,
    pub unit: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardwareIdentity {
    pub component_ref: String,
    pub hardware_uid: Option<String>,
    pub firmware_fingerprint: Option<String>,
}

/// Reference-frame definitions are not a live, post-boot joint position.
/// Unknown entries stay unknown; safe_rest_rad is not a motion authorization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointZeroReference {
    pub joint_ref: String,
    pub encoder_zero_rad: Option<f64>,
    pub mechanical_zero_rad: Option<f64>,
    pub kinematic_zero_rad: Option<f64>,
    pub initial_reference_rad: Option<f64>,
    pub safe_rest_rad: Option<f64>,
    pub homing_procedure_ref: Option<String>,
    pub evidence_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationRecord {
    pub passed: bool,
    pub method: String,
    pub validated_at: String,
    pub approver: String,
    pub evidence_ids: Vec<String>,
}

/// Ephemeral pose observation. Deliberately NOT embedded in a persisted profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionReferenceKind {
    Unknown,
    RelativeOnly,
    AbsoluteEncoder,
    VerifiedHoming,
    ExternalMeasurement,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionObservation {
    pub joint_ref: String,
    pub boot_id: String,
    pub position_rad: Option<f64>,
    pub reference_kind: PositionReferenceKind,
    pub evidence_ids: Vec<String>,
}

impl PositionObservation {
    /// Structural evidence check, not proof of sensor truth or authorization to move.
    pub fn has_current_absolute_reference(&self, current_boot_id: &str) -> bool {
        !current_boot_id.is_empty()
            && self.boot_id == current_boot_id
            && matches!(
                self.reference_kind,
                PositionReferenceKind::AbsoluteEncoder
                    | PositionReferenceKind::VerifiedHoming
                    | PositionReferenceKind::ExternalMeasurement
            )
            && self.position_rad.is_some_and(f64::is_finite)
            && !self.evidence_ids.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationProfile {
    pub schema_version: String,
    pub profile_id: String,
    pub physical_instance_id: String,
    pub model_ref: String,
    pub model_sha256: String,
    pub created_at: String,
    pub mode: CalibrationMode,
    pub state: CalibrationState,
    pub hardware: Vec<HardwareIdentity>,
    pub evidence: Vec<Evidence>,
    pub measurements: Vec<Measurement>,
    pub corrections: Vec<Correction>,
    pub joint_zero_references: Vec<JointZeroReference>,
    pub environment: BTreeMap<String, String>,
    pub validation: Option<ValidationRecord>,
    pub previous_profile_id: Option<String>,
    /// Integrity of this serialized object with this field set to null.
    pub integrity_sha256: Option<String>,
}

impl CalibrationProfile {
    pub fn new(
        profile_id: impl Into<String>,
        physical_instance_id: impl Into<String>,
        model_ref: impl Into<String>,
        model_sha256: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: PROFILE_SCHEMA_VERSION.into(),
            profile_id: profile_id.into(),
            physical_instance_id: physical_instance_id.into(),
            model_ref: model_ref.into(),
            model_sha256: model_sha256.into(),
            created_at: created_at.into(),
            mode: CalibrationMode::Passive,
            state: CalibrationState::Uncalibrated,
            hardware: Vec::new(),
            evidence: Vec::new(),
            measurements: Vec::new(),
            corrections: Vec::new(),
            joint_zero_references: Vec::new(),
            environment: BTreeMap::new(),
            validation: None,
            previous_profile_id: None,
            integrity_sha256: None,
        }
    }
}
