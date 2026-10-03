//! OpenCV pipeline will live here in later phases. Phase 3 provides just the
//! import path: decode + EXIF orientation + a JPEG preview written to the
//! session cache.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use image::{DynamicImage, ImageFormat};
use sha2::{Digest, Sha256};

use crate::filesystem::{preview_cache_dir, thumb_cache_dir};

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("decode failed: {0}")]
    Decode(#[from] image::ImageError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ImportedImage {
    pub source_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub preview_path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub source_hash: String,
}

/// Decode, apply EXIF orientation, cache a web-friendly preview and a thumb.
/// Original file is never touched.
pub fn import_from_path(src: &Path) -> Result<ImportedImage, ImageError> {
    let bytes = std::fs::read(src)?;
    let hash = hex::encode(Sha256::digest(&bytes));

    let format = image::guess_format(&bytes).ok();
    let img = match format {
        Some(fmt) => image::load_from_memory_with_format(&bytes, fmt)?,
        None => image::load_from_memory(&bytes)?,
    };
    let img = apply_exif_orientation(src, img);
    let (width, height) = (img.width(), img.height());

    let preview_path = preview_cache_dir().join(format!("{hash}.jpg"));
    if !preview_path.exists() {
        let preview = fit_within(&img, 2048);
        preview.to_rgb8().save_with_format(&preview_path, ImageFormat::Jpeg)?;
    }

    let thumbnail_path = thumb_cache_dir().join(format!("{hash}.jpg"));
    if !thumbnail_path.exists() {
        let thumb = fit_within(&img, 320);
        thumb.to_rgb8().save_with_format(&thumbnail_path, ImageFormat::Jpeg)?;
    }

    Ok(ImportedImage {
        source_path: src.to_path_buf(),
        width,
        height,
        preview_path,
        thumbnail_path,
        source_hash: hash,
    })
}

fn fit_within(img: &DynamicImage, max_side: u32) -> DynamicImage {
    let (w, h) = (img.width(), img.height());
    if w.max(h) <= max_side {
        return img.clone();
    }
    img.thumbnail(max_side, max_side)
}

/// EXIF orientation per the TIFF 6.0 spec, applied in-pixels so downstream
/// stages can forget the tag exists.
fn apply_exif_orientation(path: &Path, img: DynamicImage) -> DynamicImage {
    let Ok(file) = File::open(path) else { return img };
    let mut reader = BufReader::new(file);
    let Ok(exif) = exif::Reader::new().read_from_container(&mut reader) else {
        return img;
    };
    let Some(field) = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) else {
        return img;
    };
    let o = field.value.get_uint(0).unwrap_or(1);
    match o {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}
