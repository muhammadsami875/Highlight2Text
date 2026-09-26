use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct MagicResult {
    pub format: String,
    pub mime: String,
}

impl Default for MagicResult {
    fn default() -> Self {
        Self {
            format: "unknown".to_string(),
            mime: "application/octet-stream".to_string(),
        }
    }
}

pub fn detect_by_magic(path: &Path) -> Result<MagicResult, String> {
    let mut file = File::open(path).map_err(|e| format!("Cannot open file: {}", e))?;
    let mut buf = [0u8; 8192];
    let bytes_read = file
        .read(&mut buf)
        .map_err(|e| format!("Cannot read file: {}", e))?;
    let data = &buf[..bytes_read];

    if bytes_read < 4 {
        return Ok(MagicResult {
            format: "unknown".to_string(),
            mime: "application/octet-stream".to_string(),
        });
    }

    if data.starts_with(b"%PDF-") {
        return Ok(MagicResult {
            format: "pdf".to_string(),
            mime: "application/pdf".to_string(),
        });
    }

    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok(MagicResult {
            format: "png".to_string(),
            mime: "image/png".to_string(),
        });
    }

    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(MagicResult {
            format: "jpg".to_string(),
            mime: "image/jpeg".to_string(),
        });
    }

    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Ok(MagicResult {
            format: "webp".to_string(),
            mime: "image/webp".to_string(),
        });
    }

    if data.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
        return Ok(MagicResult {
            format: "gif".to_string(),
            mime: "image/gif".to_string(),
        });
    }

    if data.starts_with(&[0x42, 0x4D]) {
        return Ok(MagicResult {
            format: "bmp".to_string(),
            mime: "image/bmp".to_string(),
        });
    }

    if data.starts_with(&[0x49, 0x49, 0x2A, 0x00])
        || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
    {
        return Ok(MagicResult {
            format: "tiff".to_string(),
            mime: "image/tiff".to_string(),
        });
    }

    if data.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]) {
        let format = detect_ole2_subtype(path);
        let mime = match format.as_str() {
            "doc" => "application/msword",
            "xls" => "application/vnd.ms-excel",
            "ppt" => "application/vnd.ms-powerpoint",
            _ => "application/x-ole-storage",
        };
        return Ok(MagicResult {
            format,
            mime: mime.to_string(),
        });
    }

    if data.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
        let format = detect_zip_subtype(path);
        let mime = match format.as_str() {
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "odt" => "application/vnd.oasis.opendocument.text",
            "ods" => "application/vnd.oasis.opendocument.spreadsheet",
            "odp" => "application/vnd.oasis.opendocument.presentation",
            _ => "application/zip",
        };
        return Ok(MagicResult {
            format,
            mime: mime.to_string(),
        });
    }

    if data.starts_with(b"{\\rtf") {
        return Ok(MagicResult {
            format: "rtf".to_string(),
            mime: "application/rtf".to_string(),
        });
    }

    if let Ok(text) = std::str::from_utf8(&data[..bytes_read.min(4096)]) {
        let trimmed = text.trim_start();
        if trimmed.starts_with("<!DOCTYPE html")
            || trimmed.starts_with("<!doctype html")
            || trimmed.starts_with("<html")
        {
            return Ok(MagicResult {
                format: "html".to_string(),
                mime: "text/html".to_string(),
            });
        }
        if trimmed.starts_with("<?xml") || trimmed.starts_with("<xml") {
            return Ok(MagicResult {
                format: "xml".to_string(),
                mime: "application/xml".to_string(),
            });
        }
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if serde_json::from_str::<serde_json::Value>(text).is_ok() {
                return Ok(MagicResult {
                    format: "json".to_string(),
                    mime: "application/json".to_string(),
                });
            }
        }

        if looks_like_csv(text) {
            return Ok(MagicResult {
                format: "csv".to_string(),
                mime: "text/csv".to_string(),
            });
        }

        if looks_like_markdown(text) {
            return Ok(MagicResult {
                format: "md".to_string(),
                mime: "text/markdown".to_string(),
            });
        }
    }

    if let Some(kind) = infer::get(data) {
        return Ok(MagicResult {
            format: kind.extension().to_string(),
            mime: kind.mime_type().to_string(),
        });
    }

    Ok(MagicResult {
        format: "unknown".to_string(),
        mime: "application/octet-stream".to_string(),
    })
}

fn detect_zip_subtype(path: &Path) -> String {
    let Ok(file) = File::open(path) else {
        return "zip".to_string();
    };
    let Ok(mut archive) = zip::ZipArchive::new(file) else {
        return "zip".to_string();
    };

    if archive.by_name("mimetype").is_ok() {
        let mut mimetype = String::new();
        if let Ok(mut f) = archive.by_name("mimetype") {
            let _ = f.read_to_string(&mut mimetype);
        }
        return match mimetype.trim() {
            "application/vnd.oasis.opendocument.text" => "odt",
            "application/vnd.oasis.opendocument.spreadsheet" => "ods",
            "application/vnd.oasis.opendocument.presentation" => "odp",
            _ => "zip",
        }
        .to_string();
    }

    if archive.by_name("[Content_Types].xml").is_ok() {
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                let name = entry.name().to_string();
                if name.starts_with("word/") {
                    return "docx".to_string();
                }
                if name.starts_with("xl/") {
                    return "xlsx".to_string();
                }
                if name.starts_with("ppt/") {
                    return "pptx".to_string();
                }
            }
        }
    }

    "zip".to_string()
}

fn detect_ole2_subtype(_path: &Path) -> String {
    "doc".to_string()
}

fn looks_like_csv(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().take(10).collect();
    if lines.len() < 2 {
        return false;
    }

    // Only check comma and tab — semicolons cause false positives with code
    for delim in [',', '\t'] {
        let counts: Vec<usize> = lines.iter().map(|l| l.matches(delim).count()).collect();
        if counts[0] == 0 {
            continue;
        }
        let first = counts[0];
        let consistent = counts.iter().filter(|&&c| c == first).count();
        if consistent >= counts.len() * 3 / 4 {
            // Reject if lines contain braces/brackets (likely code, not CSV)
            let has_code_chars = lines.iter().any(|l| l.contains('{') || l.contains('}'));
            if !has_code_chars {
                return true;
            }
        }
    }
    false
}

fn looks_like_markdown(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().take(30).collect();
    if lines.is_empty() {
        return false;
    }
    let mut score = 0u32;

    if lines[0].starts_with("---") {
        score += 2;
    }

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") || trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            score += 2;
        }
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("1. ") {
            score += 1;
        }
        if trimmed.contains("**") || trimmed.contains("__") {
            score += 1;
        }
        if trimmed.starts_with("```") {
            score += 2;
        }
        if trimmed.starts_with('[') && trimmed.contains("](") {
            score += 2;
        }
    }

    score >= 3
}
