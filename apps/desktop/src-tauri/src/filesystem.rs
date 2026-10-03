//! Cache + per-session paths. All disk writes outside a project go here so
//! locations stay auditable.

use std::fs;
use std::path::PathBuf;

/// `%LOCALAPPDATA%\DocSnap\cache` on Windows, platform-appropriate elsewhere.
pub fn cache_root() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
    let dir = base.join("DocSnap");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn preview_cache_dir() -> PathBuf {
    let dir = cache_root().join("previews");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn thumb_cache_dir() -> PathBuf {
    let dir = cache_root().join("thumbs");
    let _ = fs::create_dir_all(&dir);
    dir
}
