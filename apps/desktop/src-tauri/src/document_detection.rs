//! Document boundary detection.
//!
//! Phase 4 ships a conservative multi-step pipeline:
//!   1. Downscale the source to a working resolution (~1024 px on the long side).
//!   2. Grayscale + Gaussian blur.
//!   3. Canny edges with auto thresholds derived from the median.
//!   4. Morphological dilate to close small gaps.
//!   5. Contour trace via imageproc.
//!   6. Keep contours with convex-hull-approx quads whose area / rectangularity /
//!      centrality score well.
//!   7. Fallback: tight bounding box of edge pixels, flagged low_confidence.
//!   8. Corners ordered TL/TR/BR/BL and mapped back to source coordinates.
//!
//! OpenCV is the long-term implementation per the Phase 1 doc; `imageproc`
//! keeps the Windows build dependency-free while the vcpkg/prebuilt decision
//! is still open.

use std::path::Path;

use image::{DynamicImage, GrayImage, Luma};
use imageproc::contours::{find_contours, BorderType, Contour};
use imageproc::edges::canny;
use imageproc::morphology::dilate_mut;
use imageproc::distance_transform::Norm;

#[derive(Debug, Clone, Copy)]
pub struct Point { pub x: f32, pub y: f32 }

#[derive(Debug, Clone)]
pub struct DocumentBoundary {
    /// TL, TR, BR, BL in source pixels.
    pub corners: [Point; 4],
    /// 0..=1. Below ~0.4 the UI should prompt manual adjustment.
    pub confidence: f32,
    pub fallback: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum DetectionError {
    #[error("image: {0}")]
    Image(#[from] image::ImageError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

const WORK_SIDE: u32 = 1024;

pub fn detect_from_path(path: &Path) -> Result<DocumentBoundary, DetectionError> {
    let img = image::open(path)?;
    Ok(detect(&img))
}

pub fn detect(src: &DynamicImage) -> DocumentBoundary {
    let (sw, sh) = (src.width() as f32, src.height() as f32);
    let scale = (WORK_SIDE as f32 / sw.max(sh)).min(1.0);
    let (ww, wh) = (((sw * scale) as u32).max(1), ((sh * scale) as u32).max(1));
    let small = src.thumbnail_exact(ww, wh).to_luma8();
    let blurred = imageproc::filter::gaussian_blur_f32(&small, 1.4);

    let (low, high) = auto_canny_thresholds(&blurred);
    let mut edges = canny(&blurred, low, high);
    dilate_mut(&mut edges, Norm::L1, 2);

    let frame_area = (ww * wh) as f32;
    let min_area = frame_area * 0.20;

    let best = find_contours::<i32>(&edges)
        .into_iter()
        .filter(|c| matches!(c.border_type, BorderType::Outer))
        .filter_map(|c| approx_quad(&c, 0.02))
        .filter(|q| polygon_area(q) >= min_area)
        .map(|q| (score_quad(&q, ww as f32, wh as f32), q))
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let (confidence, work_quad, fallback) = match best {
        Some((score, q)) if score >= 0.35 => (score.min(1.0), q, false),
        _ => (0.2, edges_bounding_quad(&edges, ww, wh), true),
    };

    let ordered = order_corners(&work_quad);
    let s = 1.0 / scale;
    let corners = [
        Point { x: ordered[0].x * s, y: ordered[0].y * s },
        Point { x: ordered[1].x * s, y: ordered[1].y * s },
        Point { x: ordered[2].x * s, y: ordered[2].y * s },
        Point { x: ordered[3].x * s, y: ordered[3].y * s },
    ];

    DocumentBoundary { corners, confidence, fallback }
}

fn auto_canny_thresholds(img: &GrayImage) -> (f32, f32) {
    let mut hist = [0u32; 256];
    for Luma([v]) in img.pixels() {
        hist[*v as usize] += 1;
    }
    let total = img.width() * img.height();
    let mut acc = 0u32;
    let mut median = 128u32;
    for (i, &c) in hist.iter().enumerate() {
        acc += c;
        if acc * 2 >= total { median = i as u32; break; }
    }
    let m = median as f32;
    ((m * 0.66).max(10.0), (m * 1.33).min(240.0))
}

fn approx_quad(contour: &Contour<i32>, epsilon_frac: f32) -> Option<[Point; 4]> {
    if contour.points.len() < 4 { return None; }
    let pts: Vec<Point> = contour
        .points
        .iter()
        .map(|p| Point { x: p.x as f32, y: p.y as f32 })
        .collect();
    let perim = polygon_perimeter(&pts);
    let eps = (perim * epsilon_frac).max(2.0);
    let approx = ramer_douglas_peucker(&pts, eps);
    if approx.len() == 4 && is_convex(&approx) {
        Some([approx[0], approx[1], approx[2], approx[3]])
    } else {
        None
    }
}

fn score_quad(q: &[Point; 4], w: f32, h: f32) -> f32 {
    let area = polygon_area(q);
    let frame = w * h;
    let area_frac = (area / frame).clamp(0.0, 1.0);

    let bb_min_x = q.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let bb_max_x = q.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
    let bb_min_y = q.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let bb_max_y = q.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
    let bb_area = (bb_max_x - bb_min_x).max(1.0) * (bb_max_y - bb_min_y).max(1.0);
    let rectangularity = (area / bb_area).clamp(0.0, 1.0);

    let cx = (q[0].x + q[1].x + q[2].x + q[3].x) * 0.25;
    let cy = (q[0].y + q[1].y + q[2].y + q[3].y) * 0.25;
    let dx = (cx - w * 0.5).abs() / (w * 0.5);
    let dy = (cy - h * 0.5).abs() / (h * 0.5);
    let centrality = (1.0 - 0.5 * (dx + dy)).clamp(0.0, 1.0);

    0.55 * area_frac + 0.30 * rectangularity + 0.15 * centrality
}

fn polygon_area(q: &[Point; 4]) -> f32 {
    let mut s = 0.0;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        s += a.x * b.y - b.x * a.y;
    }
    (s * 0.5).abs()
}

fn polygon_perimeter(pts: &[Point]) -> f32 {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let a = pts[i];
            let b = pts[(i + 1) % n];
            ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
        })
        .sum()
}

fn is_convex(pts: &[Point]) -> bool {
    let n = pts.len();
    let mut sign = 0.0;
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        let c = pts[(i + 2) % n];
        let cross = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
        if cross.abs() < f32::EPSILON { continue; }
        if sign == 0.0 { sign = cross.signum(); }
        else if cross.signum() != sign { return false; }
    }
    true
}

fn ramer_douglas_peucker(pts: &[Point], eps: f32) -> Vec<Point> {
    if pts.len() < 3 { return pts.to_vec(); }
    let mut dmax = 0.0;
    let mut index = 0;
    let end = pts.len() - 1;
    for i in 1..end {
        let d = perpendicular_distance(pts[i], pts[0], pts[end]);
        if d > dmax { dmax = d; index = i; }
    }
    if dmax > eps {
        let mut left = ramer_douglas_peucker(&pts[..=index], eps);
        let right = ramer_douglas_peucker(&pts[index..], eps);
        left.pop();
        left.extend(right);
        left
    } else {
        vec![pts[0], pts[end]]
    }
}

fn perpendicular_distance(p: Point, a: Point, b: Point) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let denom = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    ((dy * p.x - dx * p.y + b.x * a.y - b.y * a.x).abs()) / denom
}

fn edges_bounding_quad(edges: &GrayImage, w: u32, h: u32) -> [Point; 4] {
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut any = false;
    for (x, y, p) in edges.enumerate_pixels() {
        if p.0[0] > 0 {
            any = true;
            if x < min_x { min_x = x; }
            if y < min_y { min_y = y; }
            if x > max_x { max_x = x; }
            if y > max_y { max_y = y; }
        }
    }
    if !any {
        min_x = 0; min_y = 0; max_x = w.saturating_sub(1); max_y = h.saturating_sub(1);
    }
    [
        Point { x: min_x as f32, y: min_y as f32 },
        Point { x: max_x as f32, y: min_y as f32 },
        Point { x: max_x as f32, y: max_y as f32 },
        Point { x: min_x as f32, y: max_y as f32 },
    ]
}

/// Return corners in TL, TR, BR, BL order using sum/diff of coordinates.
fn order_corners(q: &[Point; 4]) -> [Point; 4] {
    let mut by_sum: Vec<(f32, Point)> = q.iter().map(|p| (p.x + p.y, *p)).collect();
    by_sum.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let tl = by_sum[0].1;
    let br = by_sum[3].1;
    let mut by_diff: Vec<(f32, Point)> = q.iter().map(|p| (p.y - p.x, *p)).collect();
    by_diff.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let tr = by_diff[0].1;
    let bl = by_diff[3].1;
    [tl, tr, br, bl]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_corners() {
        let unordered = [
            Point { x: 100.0, y: 100.0 }, // TL
            Point { x: 100.0, y: 10.0 },  // BL
            Point { x: 10.0, y: 10.0 },   // ??
            Point { x: 10.0, y: 100.0 },  // ??
        ];
        let o = order_corners(&unordered);
        assert!(o[0].x <= o[1].x && o[0].y <= o[2].y);
    }

    #[test]
    fn polygon_area_square() {
        let q = [
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
            Point { x: 0.0, y: 10.0 },
        ];
        assert!((polygon_area(&q) - 100.0).abs() < 1e-4);
    }
}
