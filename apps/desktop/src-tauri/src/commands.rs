//! Tauri command surface. All frontend-reachable IPC lives here. New phases
//! add commands; nothing else is reachable from JS.

use std::path::PathBuf;

use serde::Serialize;
use specta::Type;

use crate::document_detection::{self, DetectionError};
use crate::image_processing::{self, ImageError};
use crate::security::{self, SecurityError};

#[derive(Debug, Serialize, Type, Clone)]
pub struct ImportedPage {
    pub id: String,
    pub source_path: String,
    pub mime: String,
    pub width: u32,
    pub height: u32,
    pub byte_size: u64,
    pub source_hash: String,
    /// Absolute path to the cached JPEG preview. Convert with `convertFileSrc`
    /// on the frontend before using in an <img>.
    pub preview_path: String,
    pub thumbnail_path: String,
}

#[derive(Debug, thiserror::Error, Serialize, Type)]
#[serde(tag = "kind", content = "message")]
pub enum CommandError {
    #[error("validation: {0}")]
    Validation(String),
    #[error("image: {0}")]
    Image(String),
    #[error("io: {0}")]
    Io(String),
}

impl From<SecurityError> for CommandError {
    fn from(e: SecurityError) -> Self { CommandError::Validation(e.to_string()) }
}
impl From<ImageError> for CommandError {
    fn from(e: ImageError) -> Self { CommandError::Image(e.to_string()) }
}
impl From<DetectionError> for CommandError {
    fn from(e: DetectionError) -> Self { CommandError::Image(e.to_string()) }
}

#[derive(Debug, Serialize, Type, Clone, Copy)]
pub struct Corner { pub x: f32, pub y: f32 }

#[derive(Debug, Serialize, Type, Clone)]
pub struct DetectedBoundary {
    /// TL, TR, BR, BL in SOURCE image pixels.
    pub corners: [Corner; 4],
    pub confidence: f32,
    pub fallback: bool,
}

#[tauri::command]
#[specta::specta]
pub fn detect_document_boundary(path: String) -> Result<DetectedBoundary, CommandError> {
    let p = PathBuf::from(&path);
    let validated = security::validate_input_path(&p)?;
    let out = document_detection::detect_from_path(&validated.path)?;
    Ok(DetectedBoundary {
        corners: [
            Corner { x: out.corners[0].x, y: out.corners[0].y },
            Corner { x: out.corners[1].x, y: out.corners[1].y },
            Corner { x: out.corners[2].x, y: out.corners[2].y },
            Corner { x: out.corners[3].x, y: out.corners[3].y },
        ],
        confidence: out.confidence,
        fallback: out.fallback,
    })
}

#[tauri::command]
#[specta::specta]
pub fn import_image(path: String) -> Result<ImportedPage, CommandError> {
    let p = PathBuf::from(&path);
    let validated = security::validate_input_path(&p)?;
    let mime = security::sniff_mime(&validated.path)?;
    if !is_supported_image_mime(&mime) {
        return Err(CommandError::Validation(format!(
            "unsupported image type: {mime}"
        )));
    }
    let out = image_processing::import_from_path(&validated.path)?;
    Ok(ImportedPage {
        id: uuid::Uuid::new_v4().to_string(),
        source_path: out.source_path.to_string_lossy().into(),
        mime,
        width: out.width,
        height: out.height,
        byte_size: validated.size,
        source_hash: out.source_hash,
        preview_path: out.preview_path.to_string_lossy().into(),
        thumbnail_path: out.thumbnail_path.to_string_lossy().into(),
    })
}

fn is_supported_image_mime(mime: &str) -> bool {
    matches!(
        mime,
        "image/png"
            | "image/jpeg"
            | "image/tiff"
            | "image/bmp"
            | "image/webp"
    )
}
