use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use lopdf::Document;
use std::path::PathBuf;

pub struct PdfTextExtractor {
    manifest: ConverterManifest,
}

impl PdfTextExtractor {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "pdf_text_extract".to_string(),
                name: "PDF Text Extractor".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT".to_string(),
                input_formats: vec!["pdf".into()],
                output_formats: vec!["txt".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 100,
                capabilities: vec!["text_extraction".into()],
            },
        }
    }

    fn extract_text(path: &std::path::Path) -> Result<String, ConversionError> {
        let doc = Document::load(path)
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot load PDF: {}", e)))?;

        let mut all_text = String::new();
        let pages = doc.get_pages();
        let mut page_nums: Vec<u32> = pages.keys().copied().collect();
        page_nums.sort();

        for (i, page_num) in page_nums.iter().enumerate() {
            if i > 0 {
                all_text.push_str("\n\n--- Page ");
                all_text.push_str(&page_num.to_string());
                all_text.push_str(" ---\n\n");
            }

            let page_id = match pages.get(page_num) {
                Some(id) => *id,
                None => continue,
            };

            let content = match doc.get_page_content(page_id) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let text = extract_text_from_content(&content);
            if !text.is_empty() {
                all_text.push_str(&text);
            }
        }

        if all_text.trim().is_empty() {
            return Err(ConversionError::engine_failure(
                "pdf_text_extract",
                "No extractable text found in PDF (may be image-based)",
            ));
        }

        Ok(all_text)
    }
}

pub fn extract_pdf_text(path: &std::path::Path) -> Result<String, ConversionError> {
    PdfTextExtractor::extract_text(path)
}

fn extract_text_from_content(content: &[u8]) -> String {
    let mut result = String::new();
    let content_str = String::from_utf8_lossy(content);
    let mut in_text = false;
    let mut current_text = String::new();

    for line in content_str.lines() {
        let trimmed = line.trim();

        if trimmed == "BT" {
            in_text = true;
            current_text.clear();
            continue;
        }

        if trimmed == "ET" {
            if !current_text.is_empty() {
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(current_text.trim());
            }
            in_text = false;
            continue;
        }

        if in_text {
            if let Some(text) = extract_tj_text(trimmed) {
                current_text.push_str(&text);
            }
        }
    }

    result
}

fn extract_tj_text(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if trimmed.ends_with("Tj") {
        if let Some(start) = trimmed.find('(') {
            if let Some(end) = trimmed.rfind(')') {
                if start < end {
                    return Some(
                        trimmed[start + 1..end]
                            .replace("\\(", "(")
                            .replace("\\)", ")")
                            .replace("\\\\", "\\"),
                    );
                }
            }
        }
    }

    if trimmed.ends_with("TJ") {
        let mut text = String::new();
        let mut i = 0;
        let bytes = trimmed.as_bytes();
        while i < bytes.len() {
            if bytes[i] == b'(' {
                let mut depth = 1;
                let start = i + 1;
                i += 1;
                while i < bytes.len() && depth > 0 {
                    if bytes[i] == b'\\' {
                        i += 1;
                    } else if bytes[i] == b'(' {
                        depth += 1;
                    } else if bytes[i] == b')' {
                        depth -= 1;
                    }
                    if depth > 0 {
                        i += 1;
                    }
                }
                if i < bytes.len() {
                    let segment = String::from_utf8_lossy(&bytes[start..i]);
                    text.push_str(&segment
                        .replace("\\(", "(")
                        .replace("\\)", ")")
                        .replace("\\\\", "\\"));
                }
            }
            i += 1;
        }
        if !text.is_empty() {
            return Some(text);
        }
    }

    None
}

impl Converter for PdfTextExtractor {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        from == "pdf" && to == "txt"
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let text = Self::extract_text(&request.input_path)?;

        let stem = request
            .input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = job_dir.join("output").join(format!("{}.txt", stem));

        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;

        std::fs::write(&output_path, text)
            .map_err(|e| ConversionError::io_error(&format!("Cannot write output: {}", e)))?;

        Ok(output_path)
    }
}
