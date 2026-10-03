//! `.docsnap` project reader/writer. A `.docsnap` is a ZIP container:
//!
//! ```text
//! project.json      Project struct (versioned)
//! ocr/<pageId>.json per-page OCR results
//! ```
//!
//! Phase 16 stores recipes (corners, warp, enhancement params, OCR) and
//! references source images by canonical path. Phase 20 adds an opt-in
//! "copy sources into the project" mode so a project is portable off-machine.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use zip::write::SimpleFileOptions;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("unsupported schema version: {0}")]
    Version(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Project {
    pub schema: u32,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Page {
    pub id: String,
    pub source_path: String,
    pub source_hash: String,
    pub width: u32,
    pub height: u32,
    pub rotation: i32,
    pub corners: Option<[[f32; 2]; 4]>,
    pub warp_target: Option<[u32; 2]>,
    pub enhancement: Option<serde_json::Value>,
    pub ocr: Option<serde_json::Value>,
}

pub fn save(path: &Path, project: &Project) -> Result<(), ProjectError> {
    // Atomic rename: write .tmp first, then rename over the target.
    let tmp = path.with_extension("docsnap.tmp");
    {
        let f = File::create(&tmp)?;
        let mut zip = zip::ZipWriter::new(f);
        let opts = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("project.json", opts)?;
        zip.write_all(serde_json::to_vec_pretty(project)?.as_slice())?;
        for page in &project.pages {
            if let Some(ocr) = &page.ocr {
                zip.start_file(format!("ocr/{}.json", page.id), opts)?;
                zip.write_all(serde_json::to_vec_pretty(ocr)?.as_slice())?;
            }
        }
        zip.finish()?;
    }
    std::fs::rename(&tmp, path).map_err(ProjectError::Io)?;
    Ok(())
}

pub fn load(path: &Path) -> Result<Project, ProjectError> {
    let f = File::open(path)?;
    let mut zip = zip::ZipArchive::new(f)?;
    let mut raw = String::new();
    zip.by_name("project.json")?.read_to_string(&mut raw)?;
    let project: Project = serde_json::from_str(&raw)?;
    if project.schema > SCHEMA_VERSION {
        return Err(ProjectError::Version(project.schema));
    }
    Ok(project)
}

pub fn library_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(std::env::temp_dir);
    let dir = base.join("DocSnap").join("projects");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn list_projects() -> Vec<PathBuf> {
    let dir = library_dir();
    let mut out = vec![];
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            if e.path().extension().and_then(|s| s.to_str()) == Some("docsnap") {
                out.push(e.path());
            }
        }
    }
    out.sort_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
    out.reverse();
    out
}
