//! Explicit, bounded, read-only local OpenRGD bundle ingestion.
//! No network, driver access, device discovery, or implicit file writes.
use crate::validation::CalibrationError;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_SOURCE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct SourceDocument {
    pub role: &'static str,
    /// Relative to the caller-selected root; not a canonical model hash.
    pub path: String,
    /// SHA-256 of the exact file bytes, not the OpenRGD source-tree hash.
    pub raw_sha256: String,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub struct BundleSource {
    pub description: SourceDocument,
    pub calibration: Option<SourceDocument>,
    pub actuation_topology: Option<SourceDocument>,
    pub hal_mapping: Option<SourceDocument>,
    pub sensor_fidelity: Option<SourceDocument>,
}

impl BundleSource {
    /// Inspects a caller-selected directory in spec/, standard/, or direct
    /// 01_foundation/ layout. When both spec and standard exist, spec wins.
    /// Missing optional documents remain absent; no remote fallback is tried.
    pub fn load(root: &Path) -> Result<Self, CalibrationError> {
        let candidates = [
            (root.join("spec/01_foundation"), "spec/01_foundation"),
            (
                root.join("standard/01_foundation"),
                "standard/01_foundation",
            ),
            (root.join("01_foundation"), "01_foundation"),
            (root.to_path_buf(), "."),
        ];
        let (base, label) = candidates
            .iter()
            .find(|(base, _)| source_path(base, "description").is_some())
            .ok_or_else(|| {
                CalibrationError::Invalid(
                    "bundle requires 01_foundation/description.jsonc or .json".into(),
                )
            })?;
        let required = load_one(base, label, "description")?.ok_or_else(|| {
            CalibrationError::Invalid("description file disappeared while loading".into())
        })?;
        Ok(Self {
            description: required,
            calibration: load_one(base, label, "calibration")?,
            actuation_topology: load_one(base, label, "actuation_topology")?,
            hal_mapping: load_one(base, label, "hal_mapping")?,
            sensor_fidelity: load_one(base, label, "sensor_fidelity")?,
        })
    }
}

fn source_path(dir: &Path, role: &str) -> Option<PathBuf> {
    ["jsonc", "json"]
        .iter()
        .map(|ext| dir.join(format!("{role}.{ext}")))
        .find(|path| path.exists())
}

fn load_one(
    base: &Path,
    directory_label: &str,
    role: &'static str,
) -> Result<Option<SourceDocument>, CalibrationError> {
    let Some(path) = source_path(base, role) else {
        return Ok(None);
    };
    let metadata = fs::symlink_metadata(&path)?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_SOURCE_BYTES {
        return Err(CalibrationError::Invalid(format!(
            "not a regular file or source too large: {}",
            path.display()
        )));
    }
    let raw = fs::read(&path)?;
    let content_sha256 = format!("sha256:{:x}", Sha256::digest(&raw));
    let value: Value = if path.extension().and_then(|x| x.to_str()) == Some("jsonc") {
        serde_json::from_slice(&normalize_jsonc(&raw)?)?
    } else {
        serde_json::from_slice(&raw)?
    };
    if !value.is_object() {
        return Err(CalibrationError::Invalid(format!(
            "{role} must be a JSON object"
        )));
    }
    let relative = if directory_label == "." {
        format!(
            "{}",
            path.file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| CalibrationError::Invalid("non-UTF-8 source filename".into()))?
        )
    } else {
        format!(
            "{directory_label}/{}",
            path.file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| CalibrationError::Invalid("non-UTF-8 source filename".into()))?
        )
    };
    Ok(Some(SourceDocument {
        role,
        path: relative,
        raw_sha256: content_sha256,
        value,
    }))
}

/// Comment and trailing-comma handling outside quoted JSON strings.
/// Only bytes belonging to comments or dangling commas are changed to spaces;
/// newline locations and all actual scalar/string values are preserved.
pub fn normalize_jsonc(input: &[u8]) -> Result<Vec<u8>, CalibrationError> {
    let mut out = input.to_vec();
    let mut i = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while i < out.len() {
        let byte = out[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        if byte == b'/' && i + 1 < out.len() && out[i + 1] == b'/' {
            while i < out.len() && out[i] != b'\n' {
                out[i] = b' ';
                i += 1;
            }
            continue;
        }
        if byte == b'/' && i + 1 < out.len() && out[i + 1] == b'*' {
            let opening = i;
            out[i] = b' ';
            out[i + 1] = b' ';
            i += 2;
            let mut closed = false;
            while i < out.len() {
                if out[i] == b'*' && i + 1 < out.len() && out[i + 1] == b'/' {
                    out[i] = b' ';
                    out[i + 1] = b' ';
                    i += 2;
                    closed = true;
                    break;
                }
                if out[i] != b'\n' && out[i] != b'\r' {
                    out[i] = b' ';
                }
                i += 1;
            }
            if !closed {
                return Err(CalibrationError::Invalid(format!(
                    "unterminated JSONC block comment at byte {opening}"
                )));
            }
            continue;
        }
        i += 1;
    }
    if in_string {
        return Err(CalibrationError::Invalid(
            "unterminated JSONC string".into(),
        ));
    }
    // Separate pass so that comment whitespace is already removed.
    in_string = false;
    escaped = false;
    for i in 0..out.len() {
        match out[i] {
            b'\\' if in_string && !escaped => escaped = true,
            b'"' if !escaped => in_string = !in_string,
            b',' if !in_string => {
                let mut next = i + 1;
                while next < out.len() && out[next].is_ascii_whitespace() {
                    next += 1;
                }
                if next < out.len() && (out[next] == b']' || out[next] == b'}') {
                    out[i] = b' ';
                }
            }
            _ => escaped = false,
        }
        if out[i] == b'"' {
            escaped = false;
        }
    }
    Ok(out)
}
