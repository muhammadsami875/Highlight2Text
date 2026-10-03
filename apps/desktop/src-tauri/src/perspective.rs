//! Perspective correction.
//!
//! Given four source-image corners (TL, TR, BR, BL), compute a 3×3 homography
//! into a target rectangle whose dimensions preserve the longest edges, and
//! inverse-warp the source with bilinear sampling. OpenCV is the long-term
//! implementation (`getPerspectiveTransform` + `warpPerspective`); this pure
//! Rust version keeps the Windows build dependency-free while the Phase 1
//! vcpkg/prebuilt decision is open.

use std::path::{Path, PathBuf};

use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb, RgbImage};
use sha2::{Digest, Sha256};

use crate::document_detection::Point;
use crate::filesystem::preview_cache_dir;

/// Cap the warped output's megapixel budget. Prevents an ill-conditioned quad
/// (nearly a line) from spawning a huge buffer.
const MAX_OUTPUT_PIXELS: u64 = 24_000_000; // 24 MP

#[derive(Debug, thiserror::Error)]
pub enum PerspectiveError {
    #[error("image: {0}")]
    Image(#[from] image::ImageError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("degenerate quad")]
    Degenerate,
}

pub struct WarpResult {
    pub preview_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub warp_hash: String,
}

/// Decode the source, warp it per `corners`, cache a JPEG preview keyed by
/// `(source_hash, corners)`. The original file is never touched.
pub fn apply_from_path(src: &Path, corners: &[Point; 4]) -> Result<WarpResult, PerspectiveError> {
    let img = image::open(src)?;
    apply(&img, corners, src)
}

fn apply(img: &DynamicImage, corners: &[Point; 4], src_path: &Path) -> Result<WarpResult, PerspectiveError> {
    let (tw, th) = target_dims(corners);
    if tw < 4 || th < 4 {
        return Err(PerspectiveError::Degenerate);
    }
    let (tw, th) = clamp_budget(tw, th);

    let dst_corners = [
        Point { x: 0.0, y: 0.0 },
        Point { x: (tw - 1) as f32, y: 0.0 },
        Point { x: (tw - 1) as f32, y: (th - 1) as f32 },
        Point { x: 0.0, y: (th - 1) as f32 },
    ];
    // Inverse homography: iterate dst pixels, sample src.
    let h_inv = homography(&dst_corners, corners).ok_or(PerspectiveError::Degenerate)?;

    let rgb = img.to_rgb8();
    let (sw, sh) = (rgb.width(), rgb.height());
    let mut out: RgbImage = ImageBuffer::new(tw, th);

    for y in 0..th {
        for x in 0..tw {
            let (sx, sy) = project(&h_inv, x as f32, y as f32);
            let px = bilinear(&rgb, sw, sh, sx, sy);
            out.put_pixel(x, y, px);
        }
    }

    let mut hasher = Sha256::new();
    hasher.update(src_path.to_string_lossy().as_bytes());
    for c in corners {
        hasher.update(c.x.to_le_bytes());
        hasher.update(c.y.to_le_bytes());
    }
    let warp_hash = hex::encode(hasher.finalize());

    let preview_path = preview_cache_dir().join(format!("warp-{warp_hash}.jpg"));
    out.save_with_format(&preview_path, ImageFormat::Jpeg)?;

    Ok(WarpResult { preview_path, width: tw, height: th, warp_hash })
}

fn target_dims(c: &[Point; 4]) -> (u32, u32) {
    let w_top = dist(c[0], c[1]);
    let w_bot = dist(c[3], c[2]);
    let h_lef = dist(c[0], c[3]);
    let h_rig = dist(c[1], c[2]);
    (
        w_top.max(w_bot).round().max(1.0) as u32,
        h_lef.max(h_rig).round().max(1.0) as u32,
    )
}

fn clamp_budget(w: u32, h: u32) -> (u32, u32) {
    let pixels = (w as u64) * (h as u64);
    if pixels <= MAX_OUTPUT_PIXELS { return (w, h); }
    let scale = (MAX_OUTPUT_PIXELS as f64 / pixels as f64).sqrt();
    (
        ((w as f64) * scale).round().max(1.0) as u32,
        ((h as f64) * scale).round().max(1.0) as u32,
    )
}

fn dist(a: Point, b: Point) -> f32 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

/// Direct linear homography solve: 4 point pairs → 8-equation 8-unknown system.
/// Returns 9 coefficients row-major (h22 fixed at 1).
fn homography(src: &[Point; 4], dst: &[Point; 4]) -> Option<[f32; 9]> {
    let mut a = [[0.0f64; 8]; 8];
    let mut b = [0.0f64; 8];
    for i in 0..4 {
        let (x, y) = (src[i].x as f64, src[i].y as f64);
        let (u, v) = (dst[i].x as f64, dst[i].y as f64);
        a[i * 2] = [x, y, 1.0, 0.0, 0.0, 0.0, -u * x, -u * y];
        b[i * 2] = u;
        a[i * 2 + 1] = [0.0, 0.0, 0.0, x, y, 1.0, -v * x, -v * y];
        b[i * 2 + 1] = v;
    }
    let h = solve8(a, b)?;
    Some([
        h[0] as f32, h[1] as f32, h[2] as f32,
        h[3] as f32, h[4] as f32, h[5] as f32,
        h[6] as f32, h[7] as f32, 1.0,
    ])
}

/// Gauss-Jordan with partial pivoting for an 8×8 system. Returns `None` if
/// singular within a small tolerance.
fn solve8(mut a: [[f64; 8]; 8], mut b: [f64; 8]) -> Option<[f64; 8]> {
    let n = 8;
    for col in 0..n {
        let mut piv = col;
        let mut best = a[col][col].abs();
        for r in (col + 1)..n {
            if a[r][col].abs() > best { best = a[r][col].abs(); piv = r; }
        }
        if best < 1e-12 { return None; }
        if piv != col {
            a.swap(col, piv);
            b.swap(col, piv);
        }
        let p = a[col][col];
        for c in col..n { a[col][c] /= p; }
        b[col] /= p;
        for r in 0..n {
            if r == col { continue; }
            let f = a[r][col];
            if f == 0.0 { continue; }
            for c in col..n { a[r][c] -= f * a[col][c]; }
            b[r] -= f * b[col];
        }
    }
    Some(b)
}

fn project(h: &[f32; 9], x: f32, y: f32) -> (f32, f32) {
    let d = h[6] * x + h[7] * y + h[8];
    ((h[0] * x + h[1] * y + h[2]) / d, (h[3] * x + h[4] * y + h[5]) / d)
}

fn bilinear(img: &RgbImage, w: u32, h: u32, x: f32, y: f32) -> Rgb<u8> {
    if x < 0.0 || y < 0.0 || x > (w - 1) as f32 || y > (h - 1) as f32 {
        return Rgb([0, 0, 0]);
    }
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let dx = x - x0 as f32;
    let dy = y - y0 as f32;
    let p00 = img.get_pixel(x0, y0).0;
    let p10 = img.get_pixel(x1, y0).0;
    let p01 = img.get_pixel(x0, y1).0;
    let p11 = img.get_pixel(x1, y1).0;
    let mut out = [0u8; 3];
    for c in 0..3 {
        let v = (p00[c] as f32) * (1.0 - dx) * (1.0 - dy)
            + (p10[c] as f32) * dx * (1.0 - dy)
            + (p01[c] as f32) * (1.0 - dx) * dy
            + (p11[c] as f32) * dx * dy;
        out[c] = v.round().clamp(0.0, 255.0) as u8;
    }
    Rgb(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_quad_round_trips() {
        let src = [
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
            Point { x: 0.0, y: 10.0 },
        ];
        let dst = src;
        let h = homography(&src, &dst).unwrap();
        let (u, v) = project(&h, 5.0, 5.0);
        assert!((u - 5.0).abs() < 1e-3 && (v - 5.0).abs() < 1e-3);
    }

    #[test]
    fn target_dims_pick_longest() {
        let c = [
            Point { x: 0.0, y: 0.0 },
            Point { x: 100.0, y: 10.0 },
            Point { x: 110.0, y: 200.0 },
            Point { x: 5.0, y: 190.0 },
        ];
        let (w, h) = target_dims(&c);
        assert!(w >= 100 && h >= 189);
    }
}
