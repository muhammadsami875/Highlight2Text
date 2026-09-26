use super::manifest::ConverterManifest;
use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub trait Converter: Send + Sync {
    fn manifest(&self) -> &ConverterManifest;
    fn is_available(&self) -> bool;
    fn can_convert(&self, from: &str, to: &str) -> bool;
    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub available: bool,
    pub input_formats: Vec<String>,
    pub output_formats: Vec<String>,
    pub license: String,
}

pub struct ConverterRegistry {
    converters: Vec<Box<dyn Converter>>,
}

impl ConverterRegistry {
    pub fn new() -> Self {
        let converters: Vec<Box<dyn Converter>> = vec![
            Box::new(crate::converters::image_convert::ImageConverter::new()),
            Box::new(crate::converters::image_to_pdf::ImageToPdfConverter::new()),
            Box::new(crate::converters::markdown_to_html::MarkdownToHtmlConverter::new()),
            Box::new(crate::converters::csv_json::CsvJsonConverter::new()),
            Box::new(crate::converters::csv_xlsx::CsvXlsxConverter::new()),
            Box::new(crate::converters::code_highlight::CodeHighlightConverter::new()),
            Box::new(crate::converters::libreoffice::LibreOfficeConverter::new()),
            Box::new(crate::converters::pdf_extract::PdfTextExtractor::new()),
            Box::new(crate::converters::text_to_pdf::TextToPdfConverter::new()),
            Box::new(crate::converters::xlsx_extract::XlsxExtractConverter::new()),
            Box::new(crate::converters::html_to_pdf::HtmlToPdfConverter::new()),
            Box::new(crate::converters::pdf_to_docx::PdfToDocxConverter::new()),
            Box::new(crate::converters::pdf_to_xlsx::PdfToXlsxConverter::new()),
            Box::new(crate::converters::text_to_docx::TextToDocxConverter::new()),
        ];

        Self { converters }
    }

    pub fn find_direct(&self, from: &str, to: &str) -> Vec<&dyn Converter> {
        self.converters
            .iter()
            .filter(|c| c.is_available() && c.can_convert(from, to))
            .map(|c| c.as_ref())
            .collect()
    }

    pub fn best_converter(&self, from: &str, to: &str) -> Option<&dyn Converter> {
        let mut candidates = self.find_direct(from, to);
        candidates.sort_by(|a, b| {
            b.manifest()
                .priority
                .cmp(&a.manifest().priority)
        });
        candidates.into_iter().next()
    }

    pub fn supported_outputs(&self, from: &str) -> Vec<String> {
        let mut outputs: Vec<String> = Vec::new();
        for converter in &self.converters {
            if !converter.is_available() {
                continue;
            }
            let manifest = converter.manifest();
            if manifest.input_formats.contains(&from.to_string()) {
                for out in &manifest.output_formats {
                    if converter.can_convert(from, out) && !outputs.contains(out) {
                        outputs.push(out.clone());
                    }
                }
            }
        }
        outputs
    }

    pub fn is_supported(&self, from: &str, to: &str) -> bool {
        self.find_direct(from, to).len() > 0
    }

    pub fn engine_status(&self) -> Vec<EngineInfo> {
        self.converters
            .iter()
            .map(|c| {
                let m = c.manifest();
                EngineInfo {
                    id: m.id.clone(),
                    name: m.name.clone(),
                    version: Some(m.version.clone()),
                    available: c.is_available(),
                    input_formats: m.input_formats.clone(),
                    output_formats: m.output_formats.clone(),
                    license: m.license.clone(),
                }
            })
            .collect()
    }
}
