//! Explicit, bounded, read-only local OpenRGD bundle ingestion.
//! No network, driver access, device discovery, or implicit file writes.
use crate::validation::CalibrationError;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read};
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

fn invalid(code: &str, detail: &str) -> CalibrationError {
    CalibrationError::Invalid(format!("{code}: {detail}"))
}

fn directory_present(path: &Path) -> Result<bool, CalibrationError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_dir() => Ok(true),
        Ok(_) => Err(invalid(
            "CAL_BUNDLE_PATH",
            "source directory must be a real directory, not a symlink or special file",
        )),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

fn regular_file_present(path: &Path) -> Result<bool, CalibrationError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_file() => Ok(true),
        Ok(_) => Err(invalid(
            "CAL_BUNDLE_PATH",
            "source must be a regular file, not a symlink or special file",
        )),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

impl BundleSource {
    /// Inspect an explicit local root. spec/ wins over standard/, which is a
    /// generated mirror. Never backfill missing optional roles from another root.
    /// The directory is expected to be a stable, private snapshot, not a live
    /// directory that another process can concurrently replace.
    pub fn load(root: &Path) -> Result<Self, CalibrationError> {
        if !directory_present(root)? {
            return Err(invalid("CAL_BUNDLE_PATH", "bundle root not found"));
        }
        let candidates = [
            ("spec/01_foundation", "spec/01_foundation"),
            ("standard/01_foundation", "standard/01_foundation"),
            ("01_foundation", "01_foundation"),
            ("", "."),
        ];
        for (relative, label) in candidates {
            let mut base = root.to_path_buf();
            let mut present = true;
            for component in relative.split('/').filter(|x| !x.is_empty()) {
                base.push(component);
                if !directory_present(&base)? {
                    present = false;
                    break;
                }
            }
            if !present {
                continue;
            }
            let Some(description) = load_one(&base, label, "description")? else {
                continue;
            };
            return Ok(Self {
                description,
                calibration: load_one(&base, label, "calibration")?,
                actuation_topology: load_one(&base, label, "actuation_topology")?,
                hal_mapping: load_one(&base, label, "hal_mapping")?,
                sensor_fidelity: load_one(&base, label, "sensor_fidelity")?,
            });
        }
        Err(invalid(
            "CAL_BUNDLE_REQUIRED_FIELD",
            "bundle requires 01_foundation/description.jsonc or .json",
        ))
    }
}

fn source_path(dir: &Path, role: &str) -> Result<Option<PathBuf>, CalibrationError> {
    let jsonc = dir.join(format!("{role}.jsonc"));
    let json = dir.join(format!("{role}.json"));
    match (regular_file_present(&jsonc)?, regular_file_present(&json)?) {
        (true, true) => Err(invalid(
            "CAL_BUNDLE_AMBIGUOUS_SOURCE",
            &format!("{role}: both .json and .jsonc are present in the selected root"),
        )),
        (true, false) => Ok(Some(jsonc)),
        (false, true) => Ok(Some(json)),
        (false, false) => Ok(None),
    }
}

fn load_one(
    base: &Path,
    directory_label: &str,
    role: &'static str,
) -> Result<Option<SourceDocument>, CalibrationError> {
    let Some(path) = source_path(base, role)? else {
        return Ok(None);
    };
    let file = File::open(&path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_SOURCE_BYTES {
        return Err(invalid(
            "CAL_BUNDLE_SIZE",
            "source exceeds 4 MiB or is not regular",
        ));
    }
    let mut raw = Vec::new();
    file.take(MAX_SOURCE_BYTES + 1).read_to_end(&mut raw)?;
    if raw.len() as u64 > MAX_SOURCE_BYTES {
        return Err(invalid("CAL_BUNDLE_SIZE", "source exceeds 4 MiB"));
    }
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| invalid("CAL_BUNDLE_PATH", "non-UTF-8 source filename"))?;
    let relative = if directory_label == "." {
        filename.to_owned()
    } else {
        format!("{directory_label}/{filename}")
    };
    let value = parse_source_object(&raw, filename.ends_with(".jsonc")).map_err(|err| {
        invalid("CAL_BUNDLE_SYNTAX", &format!("{relative}: {err}; expected strict UTF-8 JSON or JSONC comments/trailing commas, not JSON5-only syntax"))
    })?;
    Ok(Some(SourceDocument {
        role,
        path: relative,
        raw_sha256: format!("sha256:{:x}", Sha256::digest(&raw)),
        value,
    }))
}

/// A recursive serde visitor rejects duplicate decoded member names (including
/// escaped aliases) before constructing any Value. Last-wins decoding would
/// make different tools disagree on the same source bytes.
struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueVisitor).map(Self)
    }
}

struct UniqueVisitor;

impl<'de> Visitor<'de> for UniqueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a finite JSON value with unique object member names")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite number"))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<UniqueValue>()? {
            values.push(value.0);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate object member"));
            }
            values.insert(key, map.next_value::<UniqueValue>()?.0);
        }
        Ok(Value::Object(values))
    }
}

/// Shared input policy for CAL-021-A. .json is strict JSON; .jsonc adds only
/// comments and a single trailing comma after a real value. JSON5-only
/// constructs are not rewritten. Raw source bytes remain the hash preimage.
pub fn parse_source_object(raw: &[u8], jsonc: bool) -> Result<Value, CalibrationError> {
    std::str::from_utf8(raw).map_err(|_| invalid("CAL_BUNDLE_UTF8", "source is not UTF-8"))?;
    let value = if jsonc {
        serde_json::from_slice::<UniqueValue>(&normalize_jsonc(raw)?)?.0
    } else {
        serde_json::from_slice::<UniqueValue>(raw)?.0
    };
    if !value.is_object() {
        return Err(invalid(
            "CAL_BUNDLE_REQUIRED_FIELD",
            "source must be a JSON object",
        ));
    }
    Ok(value)
}

/// Comment and trailing-comma handling outside quoted JSON strings. Keep all
/// offsets unchanged by replacing only comment bytes and legal trailing commas
/// with whitespace. Invalid JSON remains invalid; do not repair empty/comma-only
/// containers or missing values.
pub fn normalize_jsonc(input: &[u8]) -> Result<Vec<u8>, CalibrationError> {
    std::str::from_utf8(input).map_err(|_| invalid("CAL_BUNDLE_UTF8", "source is not UTF-8"))?;
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
            while i < out.len() && out[i] != b'\n' && out[i] != b'\r' {
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
                return Err(invalid(
                    "CAL_BUNDLE_SYNTAX",
                    &format!("unterminated JSONC block comment at byte {opening}"),
                ));
            }
            continue;
        }
        i += 1;
    }
    if in_string {
        return Err(invalid("CAL_BUNDLE_SYNTAX", "unterminated JSONC string"));
    }

    in_string = false;
    escaped = false;
    let mut previous = None;
    for i in 0..out.len() {
        let byte = out[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
                previous = Some(b'"');
            }
            continue;
        }
        if byte == b'"' {
            in_string = true;
            continue;
        }
        if byte.is_ascii_whitespace() {
            continue;
        }
        if byte == b',' && !matches!(previous, None | Some(b'{' | b'[' | b':' | b',')) {
            let next = out[i + 1..].iter().find(|x| !x.is_ascii_whitespace());
            if matches!(next, Some(b']' | b'}')) {
                out[i] = b' ';
            }
        }
        previous = Some(byte);
    }
    Ok(out)
}
