//! Tauri command surface. All frontend-reachable IPC lives here. New phases
//! add commands; nothing else is reachable from JS.

use std::path::PathBuf;

use serde::Serialize;
use specta::Type;

use crate::document_detection::{self, DetectionError, Point};
use crate::export::{self, ExportError, ImageFormatKind};
use crate::image_processing::{self, EnhancementParams, ImageError};
use crate::ocr::{self, OcrError, OcrOptions, OcrResult};
use crate::pdf::{self, ExportPage, ExportWord, PdfError, PdfExportOptions};
use crate::perspective::{self, PerspectiveError};
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
    #[error("ocr: {0}")]
    Ocr(String),
}

impl From<OcrError> for CommandError {
    fn from(e: OcrError) -> Self { CommandError::Ocr(e.to_string()) }
}
impl From<PdfError> for CommandError {
    fn from(e: PdfError) -> Self { CommandError::Io(e.to_string()) }
}
impl From<ExportError> for CommandError {
    fn from(e: ExportError) -> Self { CommandError::Io(e.to_string()) }
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
impl From<PerspectiveError> for CommandError {
    fn from(e: PerspectiveError) -> Self { CommandError::Image(e.to_string()) }
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

#[derive(Debug, Serialize, Type, Clone)]
pub struct EnhancedPage {
    pub preview_path: String,
    pub width: u32,
    pub height: u32,
    pub recipe_hash: String,
}

#[tauri::command]
#[specta::specta]
pub fn import_pdf(path: String) -> Result<Vec<ImportedPage>, CommandError> {
    let p = PathBuf::from(&path);
    let validated = security::validate_input_path(&p)?;
    let previews = pdf::import_pdf(&validated.path)?;
    Ok(previews
        .into_iter()
        .map(|pp| ImportedPage {
            id: uuid::Uuid::new_v4().to_string(),
            source_path: pp.preview_path.to_string_lossy().into(),
            mime: "image/jpeg".into(),
            width: pp.width,
            height: pp.height,
            byte_size: std::fs::metadata(&pp.preview_path).map(|m| m.len()).unwrap_or(0),
            source_hash: pp.source_hash,
            preview_path: pp.preview_path.to_string_lossy().into(),
            thumbnail_path: pp.thumbnail_path.to_string_lossy().into(),
        })
        .collect())
}

#[derive(Debug, serde::Deserialize, Type)]
pub struct PdfExportPage {
    pub image_path: String,
    pub width: u32,
    pub height: u32,
    pub words: Option<Vec<PdfExportWord>>,
}

#[derive(Debug, serde::Deserialize, Type)]
pub struct PdfExportWord {
    pub text: String,
    pub bbox: [u32; 4],
}

#[tauri::command]
#[specta::specta]
pub fn export_pdf(
    pages: Vec<PdfExportPage>,
    out_path: String,
    options: PdfExportOptions,
) -> Result<(), CommandError> {
    let converted: Vec<ExportPage> = pages
        .into_iter()
        .map(|p| ExportPage {
            image_path: PathBuf::from(p.image_path),
            width: p.width,
            height: p.height,
            words: p.words.map(|ws| {
                ws.into_iter()
                  .map(|w| ExportWord { text: w.text, bbox: w.bbox })
                  .collect()
            }),
        })
        .collect();
    pdf::export_pdf(&converted, &PathBuf::from(out_path), &options)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn export_text(out_path: String, bodies: Vec<String>) -> Result<(), CommandError> {
    export::write_text(&PathBuf::from(out_path), &bodies)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn export_image(
    source_path: String,
    out_path: String,
    kind: ImageFormatKind,
    quality: u8,
) -> Result<(), CommandError> {
    let p = PathBuf::from(&source_path);
    let validated = security::validate_input_path(&p)?;
    export::write_image(&validated.path, &PathBuf::from(out_path), kind, quality)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn run_ocr(path: String, options: OcrOptions) -> Result<OcrResult, CommandError> {
    let p = PathBuf::from(&path);
    let validated = security::validate_input_path(&p)?;
    let r = ocr::recognize(&validated.path, &options)?;
    Ok(r)
}

#[tauri::command]
#[specta::specta]
pub fn apply_enhancement(
    path: String,
    params: EnhancementParams,
) -> Result<EnhancedPage, CommandError> {
    let p = PathBuf::from(&path);
    let validated = security::validate_input_path(&p)?;
    let r = image_processing::apply_enhancement(&validated.path, &params)?;
    Ok(EnhancedPage {
        preview_path: r.preview_path.to_string_lossy().into(),
        width: r.width,
        height: r.height,
        recipe_hash: r.recipe_hash,
    })
}

#[derive(Debug, Serialize, Type, Clone)]
pub struct WarpedPage {
    pub preview_path: String,
    pub width: u32,
    pub height: u32,
    pub warp_hash: String,
}

#[tauri::command]
#[specta::specta]
pub fn apply_perspective(
    path: String,
    corners: [Corner; 4],
) -> Result<WarpedPage, CommandError> {
    let p = PathBuf::from(&path);
    let validated = security::validate_input_path(&p)?;
    let pts = [
        Point { x: corners[0].x, y: corners[0].y },
        Point { x: corners[1].x, y: corners[1].y },
        Point { x: corners[2].x, y: corners[2].y },
        Point { x: corners[3].x, y: corners[3].y },
    ];
    let r = perspective::apply_from_path(&validated.path, &pts)?;
    Ok(WarpedPage {
        preview_path: r.preview_path.to_string_lossy().into(),
        width: r.width,
        height: r.height,
        warp_hash: r.warp_hash,
    })
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
