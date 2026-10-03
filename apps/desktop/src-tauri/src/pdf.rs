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
pub struct ExportPage {
    pub image_path: PathBuf,
    pub width: u32,
    pub height: u32,
    /// Optional invisible text layer. Coordinates are in `image_path`'s pixels.
    pub words: Option<Vec<ExportWord>>,
}

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

pub fn export_pdf(
    pages: &[ExportPage],
    out: &Path,
    opts: &PdfExportOptions,
) -> Result<(), PdfError> {
    use printpdf::*;

    if pages.is_empty() {
        return Err(PdfError::Export("no pages".into()));
    }

    let (doc, first_page_idx, first_layer_idx) = PdfDocument::new(
        "DocSnap export",
        Mm(210.0),
        Mm(297.0),
        "layer",
    );

    for (i, page) in pages.iter().enumerate() {
        let (page_w_mm, page_h_mm) = page_dims_mm(page, opts.page_size);
        let (page_idx, layer_idx) = if i == 0 {
            (first_page_idx, first_layer_idx)
        } else {
            doc.add_page(Mm(page_w_mm), Mm(page_h_mm), "layer")
        };
        let layer = doc.get_page(page_idx).get_layer(layer_idx);

        let (margin_mm, inner_w, inner_h) = inner_rect(page_w_mm, page_h_mm, opts.margin);

        // Place the image to fit inside the margin box.
        let img_bytes = std::fs::read(&page.image_path)?;
        let img = image::load_from_memory(&img_bytes)?;
        let (iw, ih) = (img.width() as f32, img.height() as f32);
        let scale = (inner_w / mm_from_px(iw)).min(inner_h / mm_from_px(ih));
        let draw_w = mm_from_px(iw) * scale;
        let draw_h = mm_from_px(ih) * scale;
        let draw_x = margin_mm + (inner_w - draw_w) / 2.0;
        let draw_y = margin_mm + (inner_h - draw_h) / 2.0;

        // printpdf expects image dpi; approximate 72dpi so Mm math is linear.
        let img_obj = Image::from_dynamic_image(&img);
        img_obj.add_to_layer(layer.clone(), ImageTransform {
            translate_x: Some(Mm(draw_x)),
            translate_y: Some(Mm(draw_y)),
            scale_x: Some(scale),
            scale_y: Some(scale),
            dpi: Some(72.0),
            ..Default::default()
        });

        if opts.searchable {
            if let Some(words) = &page.words {
                let font = doc.add_builtin_font(BuiltinFont::Helvetica)
                    .map_err(|e| PdfError::Export(format!("{e:?}")))?;
                for w in words {
                    // Map pixel bbox -> page mm, within the drawn image box.
                    let x_mm = draw_x + (w.bbox[0] as f32 / iw) * draw_w;
                    let bot_mm = draw_y + (1.0 - (w.bbox[1] + w.bbox[3]) as f32 / ih) * draw_h;
                    let h_mm = (w.bbox[3] as f32 / ih) * draw_h;
                    let font_size = (h_mm * 2.83).max(1.0); // mm -> pt approx
                    layer.use_text(&w.text, font_size, Mm(x_mm), Mm(bot_mm), &font);
                }
            }
        }

        let _ = doc; // keep in scope
    }

    let mut f = std::fs::File::create(out)?;
    let mut buf = std::io::BufWriter::new(&mut f);
    doc.save(&mut buf).map_err(|e| PdfError::Export(format!("{e:?}")))?;
    Ok(())
}

fn page_dims_mm(page: &ExportPage, size: PageSize) -> (f32, f32) {
    match size {
        PageSize::A4 => (210.0, 297.0),
        PageSize::Letter => (215.9, 279.4),
        PageSize::Legal => (215.9, 355.6),
        PageSize::Original => {
            let aspect = page.height as f32 / page.width.max(1) as f32;
            (210.0, (210.0 * aspect).clamp(50.0, 1500.0))
        }
    }
}

fn inner_rect(w: f32, h: f32, m: Margin) -> (f32, f32, f32) {
    let margin = match m { Margin::None => 0.0, Margin::Small => 5.0, Margin::Normal => 15.0 };
    (margin, w - 2.0 * margin, h - 2.0 * margin)
}

fn mm_from_px(px: f32) -> f32 { px * 25.4 / 72.0 }
