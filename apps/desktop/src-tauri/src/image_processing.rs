//! Decode / orient / cache + enhancement pipeline.
//!
//! OpenCV remains the long-term plan per the Phase 1 doc. This module uses the
//! `image` + `imageproc` crates so the Windows build stays dependency-free
//! until the vcpkg decision closes.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use image::{DynamicImage, ImageFormat};
use imageproc::contrast::{adaptive_threshold, otsu_level, threshold};
use imageproc::contrast::ThresholdType;
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
        fit_within(&img, 2048).to_rgb8().save_with_format(&preview_path, ImageFormat::Jpeg)?;
    }
    let thumbnail_path = thumb_cache_dir().join(format!("{hash}.jpg"));
    if !thumbnail_path.exists() {
        fit_within(&img, 320).to_rgb8().save_with_format(&thumbnail_path, ImageFormat::Jpeg)?;
    }

    Ok(ImportedImage { source_path: src.to_path_buf(), width, height, preview_path, thumbnail_path, source_hash: hash })
}

fn fit_within(img: &DynamicImage, max_side: u32) -> DynamicImage {
    let (w, h) = (img.width(), img.height());
    if w.max(h) <= max_side { return img.clone(); }
    img.thumbnail(max_side, max_side)
}

fn apply_exif_orientation(path: &Path, img: DynamicImage) -> DynamicImage {
    let Ok(file) = File::open(path) else { return img };
    let mut reader = BufReader::new(file);
    let Ok(exif) = exif::Reader::new().read_from_container(&mut reader) else { return img };
    let Some(field) = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) else { return img };
    match field.value.get_uint(0).unwrap_or(1) {
        2 => img.fliph(), 3 => img.rotate180(), 4 => img.flipv(),
        5 => img.rotate90().fliph(), 6 => img.rotate90(),
        7 => img.rotate270().fliph(), 8 => img.rotate270(), _ => img,
    }
}

// --- Phase 7: enhancement -------------------------------------------------

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, specta::Type)]
pub enum Preset {
    Original, Color, Auto, Document, BlackWhite, Grayscale, HighContrast,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, specta::Type)]
pub struct EnhancementParams {
    pub preset: Preset,
    /// -100..=100
    pub brightness: i32,
    /// -100..=100
    pub contrast: i32,
    /// 0..=100
    pub sharpness: i32,
    /// 0..=100 — unsharp amount for shadow removal (document background flatten).
    pub shadow_remove: i32,
}

pub struct EnhancementResult {
    pub preview_path: PathBuf,
    pub recipe_hash: String,
    pub width: u32,
    pub height: u32,
}

pub fn apply_enhancement(src: &Path, params: &EnhancementParams) -> Result<EnhancementResult, ImageError> {
    let img = image::open(src)?;
    let working = enhance(&img, params);

    let mut hasher = Sha256::new();
    hasher.update(src.to_string_lossy().as_bytes());
    hasher.update(serde_json::to_vec(params).unwrap_or_default());
    let recipe_hash = hex::encode(hasher.finalize());

    let preview_path = preview_cache_dir().join(format!("enh-{recipe_hash}.jpg"));
    let (w, h) = (working.width(), working.height());
    working.to_rgb8().save_with_format(&preview_path, ImageFormat::Jpeg)?;
    Ok(EnhancementResult { preview_path, recipe_hash, width: w, height: h })
}

fn enhance(img: &DynamicImage, p: &EnhancementParams) -> DynamicImage {
    // Preset first; sliders are applied after on the color/gray result so they
    // compose predictably.
    let base = match p.preset {
        Preset::Original => img.clone(),
        Preset::Color => auto_white_balance(img),
        Preset::Auto => clahe_like(img),
        Preset::Document => document_threshold(img),
        Preset::BlackWhite => otsu(img),
        Preset::Grayscale => DynamicImage::ImageLuma8(img.to_luma8()),
        Preset::HighContrast => s_curve(&clahe_like(img), 1.6),
    };
    let mut out = base;
    if p.brightness != 0 { out = adjust_brightness(&out, p.brightness); }
    if p.contrast != 0 { out = adjust_contrast(&out, p.contrast); }
    if p.shadow_remove > 0 { out = flatten_background(&out, p.shadow_remove); }
    if p.sharpness > 0 { out = sharpen(&out, p.sharpness); }
    out
}

fn auto_white_balance(img: &DynamicImage) -> DynamicImage {
    let mut rgb = img.to_rgb8();
    // Simple grey-world: scale each channel so its mean matches the global mean.
    let mut sum = [0u64; 3];
    let n = (rgb.width() * rgb.height()) as u64;
    for p in rgb.pixels() {
        for c in 0..3 { sum[c] += p.0[c] as u64; }
    }
    let mean = (sum[0] + sum[1] + sum[2]) as f32 / (3.0 * n as f32);
    let factors = [
        if sum[0] == 0 { 1.0 } else { mean / (sum[0] as f32 / n as f32) },
        if sum[1] == 0 { 1.0 } else { mean / (sum[1] as f32 / n as f32) },
        if sum[2] == 0 { 1.0 } else { mean / (sum[2] as f32 / n as f32) },
    ];
    for p in rgb.pixels_mut() {
        for c in 0..3 {
            p.0[c] = ((p.0[c] as f32 * factors[c]).clamp(0.0, 255.0)) as u8;
        }
    }
    DynamicImage::ImageRgb8(rgb)
}

fn clahe_like(img: &DynamicImage) -> DynamicImage {
    // Global histogram equalization on the luminance channel; cheap approximation
    // of CLAHE while OpenCV isn't wired.
    let luma = img.to_luma8();
    let equalized = imageproc::contrast::equalize_histogram(&luma);
    // Recombine with original chroma via simple ratio.
    let rgb = img.to_rgb8();
    let mut out = image::RgbImage::new(rgb.width(), rgb.height());
    for (x, y, src) in rgb.enumerate_pixels() {
        let y_old = (0.299 * src.0[0] as f32 + 0.587 * src.0[1] as f32 + 0.114 * src.0[2] as f32).max(1.0);
        let y_new = equalized.get_pixel(x, y).0[0] as f32;
        let ratio = y_new / y_old;
        let mut p = [0u8; 3];
        for c in 0..3 { p[c] = ((src.0[c] as f32 * ratio).clamp(0.0, 255.0)) as u8; }
        out.put_pixel(x, y, image::Rgb(p));
    }
    DynamicImage::ImageRgb8(out)
}

fn document_threshold(img: &DynamicImage) -> DynamicImage {
    let luma = img.to_luma8();
    let bin = adaptive_threshold(&luma, 15);
    DynamicImage::ImageLuma8(bin)
}

fn otsu(img: &DynamicImage) -> DynamicImage {
    let luma = img.to_luma8();
    let level = otsu_level(&luma);
    let bin = threshold(&luma, level, ThresholdType::Binary);
    DynamicImage::ImageLuma8(bin)
}

fn s_curve(img: &DynamicImage, strength: f32) -> DynamicImage {
    let mut rgb = img.to_rgb8();
    for p in rgb.pixels_mut() {
        for c in 0..3 {
            let v = p.0[c] as f32 / 255.0;
            let t = 1.0 / (1.0 + (-strength * (v - 0.5) * 4.0).exp());
            p.0[c] = (t * 255.0).clamp(0.0, 255.0) as u8;
        }
    }
    DynamicImage::ImageRgb8(rgb)
}

fn adjust_brightness(img: &DynamicImage, delta: i32) -> DynamicImage {
    let shift = ((delta.clamp(-100, 100)) as f32) * 2.55;
    let mut rgb = img.to_rgb8();
    for p in rgb.pixels_mut() {
        for c in 0..3 { p.0[c] = (p.0[c] as f32 + shift).clamp(0.0, 255.0) as u8; }
    }
    DynamicImage::ImageRgb8(rgb)
}

fn adjust_contrast(img: &DynamicImage, delta: i32) -> DynamicImage {
    let factor = (259.0 * (delta as f32 + 255.0)) / (255.0 * (259.0 - delta as f32));
    let mut rgb = img.to_rgb8();
    for p in rgb.pixels_mut() {
        for c in 0..3 {
            let v = (factor * (p.0[c] as f32 - 128.0) + 128.0).clamp(0.0, 255.0);
            p.0[c] = v as u8;
        }
    }
    DynamicImage::ImageRgb8(rgb)
}

fn flatten_background(img: &DynamicImage, strength: i32) -> DynamicImage {
    // Morphological background estimation: large-kernel blur approximates the
    // low-frequency illumination field; divide the source by it and renormalize.
    let s = (strength.clamp(1, 100) as f32) / 100.0;
    let blur_sigma = 20.0 + 60.0 * s;
    let luma = img.to_luma8();
    let background = imageproc::filter::gaussian_blur_f32(&luma, blur_sigma);
    let (w, h) = (luma.width(), luma.height());
    let rgb = img.to_rgb8();
    let mut out = image::RgbImage::new(w, h);
    for (x, y, p) in rgb.enumerate_pixels() {
        let bg = background.get_pixel(x, y).0[0] as f32 + 1.0;
        let scale = 220.0 / bg; // target background ~220
        let mut r = [0u8; 3];
        for c in 0..3 { r[c] = ((p.0[c] as f32 * scale).clamp(0.0, 255.0)) as u8; }
        out.put_pixel(x, y, image::Rgb(r));
    }
    DynamicImage::ImageRgb8(out)
}

fn sharpen(img: &DynamicImage, amount: i32) -> DynamicImage {
    let a = (amount.clamp(0, 100) as f32) / 100.0;
    let rgb = img.to_rgb8();
    let blurred = imageproc::filter::gaussian_blur_f32(&rgb, 1.2);
    let (w, h) = (rgb.width(), rgb.height());
    let mut out = image::RgbImage::new(w, h);
    for (x, y, p) in rgb.enumerate_pixels() {
        let b = blurred.get_pixel(x, y).0;
        let mut r = [0u8; 3];
        for c in 0..3 {
            let hp = p.0[c] as f32 - b[c] as f32;
            r[c] = (p.0[c] as f32 + a * hp).clamp(0.0, 255.0) as u8;
        }
        out.put_pixel(x, y, image::Rgb(r));
    }
    DynamicImage::ImageRgb8(out)
}
