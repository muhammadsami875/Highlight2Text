//! PDF import (pdfium-render) and export (printpdf). Pages are rendered to
//! JPEG previews on demand; nothing holds the whole document in RAM.

use std::path::{Path, PathBuf};

use image::ImageFormat;
use pdfium_render::prelude::*;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::filesystem::{preview_cache_dir, thumb_cache_dir};

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("pdfium not found; place pdfium.dll next to the executable")]
    PdfiumMissing,
    #[error("pdfium: {0}")]
    Pdfium(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("image: {0}")]
    Image(#[from] image::ImageError),
    #[error("pdf export: {0}")]
    Export(String),
}

impl From<PdfiumError> for PdfError {
    fn from(e: PdfiumError) -> Self { PdfError::Pdfium(format!("{e:?}")) }
}

#[allow(dead_code)]
pub struct PdfPagePreview {
    pub index: u32,
    pub preview_path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub source_hash: String,
}

/// Render every page in `src` to cached JPEG previews. Caller decides how many
/// of these to materialize as ImportedPages.
pub fn import_pdf(src: &Path) -> Result<Vec<PdfPagePreview>, PdfError> {
    let pdfium = load_pdfium()?;
    let doc = pdfium.load_pdf_from_file(src, None)?;
    let n = doc.pages().len();

    let bytes_hash = {
        let bytes = std::fs::read(src)?;
        hex::encode(Sha256::digest(&bytes))
    };

    let mut out = Vec::with_capacity(n as usize);
    for i in 0..n {
        let page = doc.pages().get(i)?;
        let cfg = PdfRenderConfig::new()
            .set_target_width(2048)
            .set_maximum_height(2048);
        let img = page.render_with_config(&cfg)?.as_image().to_rgb8();
        let (w, h) = (img.width(), img.height());

        let preview_path = preview_cache_dir()
            .join(format!("pdf-{bytes_hash}-{i:04}.jpg"));
        if !preview_path.exists() {
            img.save_with_format(&preview_path, ImageFormat::Jpeg)?;
        }
        let thumb = image::imageops::thumbnail(&img, 320, 320);
        let thumbnail_path = thumb_cache_dir()
            .join(format!("pdf-{bytes_hash}-{i:04}.jpg"));
        if !thumbnail_path.exists() {
            thumb.save_with_format(&thumbnail_path, ImageFormat::Jpeg)?;
        }
        out.push(PdfPagePreview {
            index: i as u32,
            preview_path,
            thumbnail_path,
            width: w,
            height: h,
            source_hash: format!("{bytes_hash}-{i:04}"),
        });
    }
    Ok(out)
}

fn load_pdfium() -> Result<Pdfium, PdfError> {
    // 1. Next to the executable (production bundle layout).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = Pdfium::pdfium_platform_library_name_at_path(dir);
            if let Ok(b) = Pdfium::bind_to_library(&candidate) {
                return Ok(Pdfium::new(b));
            }
        }
    }
    // 2. System library on PATH / loader paths.
    if let Ok(b) = Pdfium::bind_to_system_library() {
        return Ok(Pdfium::new(b));
    }
    Err(PdfError::PdfiumMissing)
}

// -------------------------------------------------------------------------
// Export — see Phase 12 doc.

/// A single page to write into the output PDF.
#[allow(dead_code)]
pub struct ExportPage {
    pub image_path: PathBuf,
    pub width: u32,
    pub height: u32,
    /// Optional invisible text layer. Coordinates are in `image_path`'s pixels.
    pub words: Option<Vec<ExportWord>>,
}

#[allow(dead_code)]
pub struct ExportWord {
    pub text: String,
    pub bbox: [u32; 4],
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, specta::Type)]
pub enum PageSize { A4, Letter, Legal, Original }

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, specta::Type)]
pub enum Margin { None, Small, Normal }

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, specta::Type)]
pub enum Quality { High, Balanced, Small }

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, specta::Type)]
pub struct PdfExportOptions {
    pub page_size: PageSize,
    pub margin: Margin,
    pub quality: Quality,
    pub searchable: bool,
}

/// Phase 12 scaffolded this with `printpdf`; the actual encoder wiring is
/// deliberately deferred to the Phase 20 bundling spike so a specific
/// `printpdf` patch release can be pinned against real documents. The
/// command surface stays stable; a release build swaps this stub for the
/// real impl without touching the frontend.
pub fn export_pdf(
    _pages: &[ExportPage],
    _out: &Path,
    _opts: &PdfExportOptions,
) -> Result<(), PdfError> {
    Err(PdfError::Export(
        "PDF export is not yet wired to a pinned printpdf release; see docs/PHASE_12_13_EXPORT.md".into(),
    ))
}
