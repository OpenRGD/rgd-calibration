use crate::model::CalibrationProfile;
use crate::validation::{validate_profile_id, verify, CalibrationError};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct FileStore {
    root: PathBuf,
}

impl FileStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn location(&self, id: &str) -> Result<PathBuf, CalibrationError> {
        validate_profile_id(id)?;
        Ok(self.root.join("profiles").join(format!("{id}.json")))
    }

    /// Create an immutable, sealed revision. Never overwrite a previous profile.
    /// Uses same-directory temporary file and no-clobber atomic persistence.
    pub fn save_new(&self, profile: &CalibrationProfile) -> Result<PathBuf, CalibrationError> {
        verify(profile)?;
        let destination = self.location(&profile.profile_id)?;
        let directory = destination.parent().ok_or_else(|| {
            CalibrationError::Invalid("profile store directory missing".into())
        })?;
        fs::create_dir_all(directory)?;
        let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
        serde_json::to_writer_pretty(temporary.as_file_mut(), profile)?;
        temporary.as_file_mut().write_all(b"\n")?;
        temporary.as_file_mut().sync_all()?;
        temporary
            .persist_noclobber(&destination)
            .map_err(|error| CalibrationError::Io(error.error))?;
        Ok(destination)
    }

    pub fn load(&self, profile_id: &str) -> Result<CalibrationProfile, CalibrationError> {
        let location = self.location(profile_id)?;
        let profile: CalibrationProfile = serde_json::from_slice(&fs::read(location)?)?;
        if profile.profile_id != profile_id {
            return Err(CalibrationError::Invalid(
                "profile id differs from the stored filename".into(),
            ));
        }
        verify(&profile)?;
        Ok(profile)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}
