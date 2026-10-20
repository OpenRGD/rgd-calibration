//! Operator-invoked, strictly passive JSON report CLI.
//!
//! Usage:
//!   rgd-calibration inspect --bundle PATH
//!       [--profile FILE] [--instance ID] [--model-hash sha256:...]
//!       [--out FILE]
use rgd_calibration::passive::{inspect_passive, InspectionContext};
use rgd_calibration::source::BundleSource;
use rgd_calibration::{verify, CalibrationError, CalibrationProfile};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const MAX_PROFILE_BYTES: u64 = 4 * 1024 * 1024;

fn input_error(message: &str) -> CalibrationError {
    CalibrationError::Invalid(message.to_owned())
}

fn read_profile(path: &Path) -> Result<CalibrationProfile, CalibrationError> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.file_type().is_file() || meta.len() > MAX_PROFILE_BYTES {
        return Err(input_error("profile must be a regular JSON file <= 4 MiB"));
    }
    let profile: CalibrationProfile = serde_json::from_slice(&fs::read(path)?)?;
    verify(&profile)?;
    Ok(profile)
}

fn usage() -> &'static str {
    "Usage: rgd-calibration inspect --bundle ROOT [--profile FILE] [--instance ID] [--model-hash sha256:HEX64] [--out FILE]\n\
     Reports are passive: no network, no drivers, no motion, no calibration commits.\n\
     --model-hash must be an externally established model digest; raw input file SHA-256 is not a substitute."
}

fn run() -> Result<(), CalibrationError> {
    let mut args = env::args().skip(1);
    if args.next().as_deref() != Some("inspect") {
        return Err(input_error(usage()));
    }
    let mut bundle: Option<PathBuf> = None;
    let mut profile_path: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut context = InspectionContext::default();
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| input_error("missing option value"))?;
        if value.starts_with("--") {
            return Err(input_error("option values must not begin with --"));
        }
        match flag.as_str() {
            "--bundle" if bundle.is_none() => bundle = Some(PathBuf::from(value)),
            "--profile" if profile_path.is_none() => profile_path = Some(PathBuf::from(value)),
            "--instance" if context.physical_instance_id.is_none() => {
                context.physical_instance_id = Some(value)
            }
            "--model-hash" if context.model_sha256.is_none() => {
                let valid = value
                    .strip_prefix("sha256:")
                    .is_some_and(|hex| {
                        hex.len() == 64 && hex.bytes().all(|x| x.is_ascii_hexdigit())
                    });
                if !valid {
                    return Err(input_error("--model-hash requires sha256:<64 hex digits>"));
                }
                context.model_sha256 = Some(value);
            }
            "--out" if out.is_none() => out = Some(PathBuf::from(value)),
            _ => return Err(input_error("unknown or repeated option")),
        }
    }
    let root = bundle.ok_or_else(|| input_error(usage()))?;
    let bundle = BundleSource::load(&root)?;
    let profile = profile_path
        .as_deref()
        .map(read_profile)
        .transpose()?;
    let report = inspect_passive(&bundle, profile.as_ref(), &context)?;
    let mut json = serde_json::to_vec_pretty(&report)?;
    json.push(b'\n');
    if let Some(destination) = out {
        let mut writer = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)?;
        writer.write_all(&json)?;
        writer.sync_all()?;
    } else {
        std::io::stdout().write_all(&json)?;
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("CAL-0.2 inspection error: {error}");
        std::process::exit(2);
    }
}
