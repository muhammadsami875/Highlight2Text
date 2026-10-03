//! OCR sidecar manager.
//!
//! Tesseract runs as a child process. For the bundled builds, the binary lives
//! under `resources/binaries/` and `tessdata/` holds language packs; during
//! `tauri dev` we fall back to a system Tesseract on PATH so engineers don't
//! need the full packaging rig.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use thiserror::Error;

use crate::filesystem::cache_root;

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("tesseract not found on PATH; install it or ship it in resources/binaries")]
    EngineMissing,
    #[error("engine exited {code:?}: {stderr}")]
    EngineFailed { code: Option<i32>, stderr: String },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid utf-8 from engine")]
    Utf8,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, specta::Type)]
pub enum PageMode { Auto, SingleBlock, MultiBlock, SingleLine, SparseText, Table }

impl PageMode {
    fn psm(self) -> u32 {
        match self { Self::Auto => 3, Self::SingleBlock => 6, Self::MultiBlock => 1,
            Self::SingleLine => 7, Self::SparseText => 11, Self::Table => 4 }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, specta::Type)]
pub struct OcrOptions {
    /// Tesseract language code(s), e.g. "eng" or "eng+deu".
    pub languages: String,
    pub mode: PageMode,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct OcrWord {
    pub text: String,
    pub bbox: [u32; 4], // x, y, w, h
    pub confidence: f32, // 0..=1 — only when Tesseract reports it
    pub line: u32,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct OcrResult {
    pub text: String,
    pub words: Vec<OcrWord>,
    pub languages: String,
}

pub fn recognize(image_path: &std::path::Path, opts: &OcrOptions) -> Result<OcrResult, OcrError> {
    let tess = find_tesseract()?;
    let tmp_dir = cache_root().join("ocr");
    std::fs::create_dir_all(&tmp_dir)?;
    let stem = tmp_dir.join(format!("ocr-{}", uuid::Uuid::new_v4()));
    let txt_out = stem.with_extension("txt");
    let tsv_out = stem.with_extension("tsv");

    // Run Tesseract twice: once for plain text, once for TSV with word boxes.
    // Avoids relying on a single config profile that may be missing.
    run_tesseract(&tess, image_path, &stem, opts, "txt")?;
    run_tesseract(&tess, image_path, &stem, opts, "tsv")?;

    let text = std::fs::read_to_string(&txt_out).unwrap_or_default();
    let words = parse_tsv(&std::fs::read_to_string(&tsv_out).unwrap_or_default());

    // Best-effort cleanup; errors here aren't user-visible failures.
    let _ = std::fs::remove_file(&txt_out);
    let _ = std::fs::remove_file(&tsv_out);

    Ok(OcrResult { text, words, languages: opts.languages.clone() })
}

fn run_tesseract(
    tess: &PathBuf,
    image: &std::path::Path,
    stem: &std::path::Path,
    opts: &OcrOptions,
    config: &str,
) -> Result<(), OcrError> {
    let mut cmd = Command::new(tess);
    cmd.arg(image).arg(stem)
       .arg("-l").arg(&opts.languages)
       .arg("--psm").arg(opts.mode.psm().to_string())
       .arg(config)
       .stdout(Stdio::piped()).stderr(Stdio::piped());
    // Point Tesseract at bundled tessdata if present.
    if let Some(tessdata) = bundled_tessdata_dir() {
        cmd.env("TESSDATA_PREFIX", tessdata);
    }
    let out = cmd.output()?;
    if !out.status.success() {
        let mut stderr = String::from_utf8_lossy(&out.stderr).to_string();
        if stderr.is_empty() { stderr = "tesseract failed with no output".into(); }
        let _ = std::io::stderr().write_all(&out.stderr);
        return Err(OcrError::EngineFailed { code: out.status.code(), stderr });
    }
    Ok(())
}

fn find_tesseract() -> Result<PathBuf, OcrError> {
    // Prefer the bundled sidecar; fall back to PATH for dev.
    #[cfg(target_os = "windows")]
    let bundled = resource_path("binaries/tesseract.exe");
    #[cfg(not(target_os = "windows"))]
    let bundled = resource_path("binaries/tesseract");

    if let Some(p) = bundled {
        if p.exists() { return Ok(p); }
    }
    which::which("tesseract").map_err(|_| OcrError::EngineMissing)
}

fn resource_path(rel: &str) -> Option<PathBuf> {
    // Best-effort. In production tauri provides a resolved resource dir; here
    // we check next to the executable.
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.to_path_buf();
    Some(dir.join("resources").join(rel))
}

fn bundled_tessdata_dir() -> Option<PathBuf> {
    let p = resource_path("tessdata")?;
    p.exists().then_some(p)
}

/// Parse Tesseract's TSV: cols include `level, page, block, par, line, word,
/// left, top, width, height, conf, text`.
fn parse_tsv(tsv: &str) -> Vec<OcrWord> {
    let mut out = Vec::new();
    let mut lines = tsv.lines();
    let header = lines.next().unwrap_or("");
    let cols: Vec<&str> = header.split('\t').collect();
    let idx = |n: &str| cols.iter().position(|c| *c == n);
    let (i_level, i_line, i_x, i_y, i_w, i_h, i_conf, i_text) = (
        idx("level"), idx("line_num"), idx("left"), idx("top"),
        idx("width"), idx("height"), idx("conf"), idx("text"),
    );
    for row in lines {
        let f: Vec<&str> = row.split('\t').collect();
        let get = |i: Option<usize>| i.and_then(|j| f.get(j)).copied();
        let Some("5") = get(i_level) else { continue }; // word-level rows
        let text = get(i_text).unwrap_or("").trim();
        if text.is_empty() { continue; }
        let line = get(i_line).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let x = get(i_x).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let y = get(i_y).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let w = get(i_w).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let h = get(i_h).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let conf = get(i_conf).and_then(|s| s.parse::<f32>().ok()).unwrap_or(-1.0);
        out.push(OcrWord {
            text: text.to_string(),
            bbox: [x, y, w, h],
            confidence: if conf < 0.0 { 0.0 } else { (conf / 100.0).clamp(0.0, 1.0) },
            line,
        });
    }
    out
}
