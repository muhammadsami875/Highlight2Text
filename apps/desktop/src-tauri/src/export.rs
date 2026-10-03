//! Text and image exporters. PDF export lives in `pdf.rs`.

use std::path::Path;

use image::ImageFormat;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("image: {0}")]
    Image(#[from] image::ImageError),
    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, specta::Type)]
pub enum TextFormat { Txt, Json, Csv }

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, specta::Type)]
pub enum ImageFormatKind { Png, Jpeg, Webp, Tiff }

// JsonPage / JsonWord shapes are reserved for Phase 13's JSON exporter once
// it has UI; kept out of the compiled surface until then.

pub fn write_text(out: &Path, bodies: &[String]) -> Result<(), ExportError> {
    let joined = bodies.join("\n\n\u{000C}\n\n"); // form-feed between pages
    std::fs::write(out, joined)?;
    Ok(())
}

pub fn write_image(src: &Path, out: &Path, kind: ImageFormatKind, quality: u8) -> Result<(), ExportError> {
    let img = image::open(src)?;
    let fmt = match kind {
        ImageFormatKind::Png => ImageFormat::Png,
        ImageFormatKind::Jpeg => ImageFormat::Jpeg,
        ImageFormatKind::Webp => ImageFormat::WebP,
        ImageFormatKind::Tiff => ImageFormat::Tiff,
    };
    if matches!(fmt, ImageFormat::Jpeg) {
        use image::codecs::jpeg::JpegEncoder;
        let mut f = std::fs::File::create(out)?;
        let mut enc = JpegEncoder::new_with_quality(&mut f, quality.max(1).min(100));
        enc.encode_image(&img)?;
    } else {
        img.save_with_format(out, fmt)?;
    }
    Ok(())
}
