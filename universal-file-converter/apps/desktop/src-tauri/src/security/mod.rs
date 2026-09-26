use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum SecurityError {
    PathTraversal(String),
    FileTooLarge { size: u64, max: u64 },
    UnsafePath(String),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::PathTraversal(p) => write!(f, "Path traversal detected: {}", p),
            SecurityError::FileTooLarge { size, max } => {
                write!(f, "File too large: {} bytes (max {})", size, max)
            }
            SecurityError::UnsafePath(p) => write!(f, "Unsafe path: {}", p),
        }
    }
}

pub fn sanitize_path(input: &str, allowed_base: &Path) -> Result<PathBuf, SecurityError> {
    let path = PathBuf::from(input);

    let canonical = path.canonicalize().map_err(|_| {
        SecurityError::UnsafePath(format!("Cannot resolve path: {}", input))
    })?;

    if !canonical.starts_with(allowed_base) {
        return Err(SecurityError::PathTraversal(format!(
            "Path {} is outside allowed directory {}",
            canonical.display(),
            allowed_base.display()
        )));
    }

    Ok(canonical)
}

pub fn validate_file_size(path: &Path, max_bytes: u64) -> Result<u64, SecurityError> {
    let metadata = std::fs::metadata(path).map_err(|_| {
        SecurityError::UnsafePath(format!("Cannot read file metadata: {}", path.display()))
    })?;

    let size = metadata.len();
    if size > max_bytes {
        return Err(SecurityError::FileTooLarge {
            size,
            max: max_bytes,
        });
    }

    Ok(size)
}

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim_start_matches('.')
        .to_string()
}
