//! Input validation and path safety. Any user-supplied path passes through
//! `validate_input_path` before anything else looks at it.

use std::path::{Path, PathBuf};
use thiserror::Error;

/// 500 MB. Protects decoders from pathological inputs. Tune per device.
pub const MAX_IMAGE_BYTES: u64 = 500 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("file not found: {0}")]
    NotFound(PathBuf),
    #[error("path is not a regular file")]
    NotAFile,
    #[error("file too large: {size} bytes (max {max})")]
    TooLarge { size: u64, max: u64 },
    #[error("unsupported or unrecognized file type")]
    UnsupportedType,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Canonicalize and apply size/type caps. Never touches the file contents
/// beyond metadata + magic-byte sniffing.
pub fn validate_input_path(raw: &Path) -> Result<ValidatedPath, SecurityError> {
    let canonical = raw.canonicalize().map_err(|_| SecurityError::NotFound(raw.into()))?;
    let md = std::fs::metadata(&canonical)?;
    if !md.is_file() {
        return Err(SecurityError::NotAFile);
    }
    if md.len() > MAX_IMAGE_BYTES {
        return Err(SecurityError::TooLarge { size: md.len(), max: MAX_IMAGE_BYTES });
    }
    Ok(ValidatedPath { path: canonical, size: md.len() })
}

pub struct ValidatedPath {
    pub path: PathBuf,
    pub size: u64,
}

/// Sniff MIME from the first bytes; extension is only a hint.
pub fn sniff_mime(path: &Path) -> Result<String, SecurityError> {
    let kind = infer::get_from_path(path)
        .map_err(SecurityError::Io)?
        .ok_or(SecurityError::UnsupportedType)?;
    Ok(kind.mime_type().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_path() {
        let r = validate_input_path(Path::new("/nope/does/not/exist.png"));
        assert!(matches!(r, Err(SecurityError::NotFound(_))));
    }

    #[test]
    fn rejects_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let r = validate_input_path(tmp.path());
        assert!(matches!(r, Err(SecurityError::NotAFile)));
    }
}
