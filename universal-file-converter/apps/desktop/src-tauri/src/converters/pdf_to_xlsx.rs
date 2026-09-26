use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use std::path::PathBuf;

pub struct PdfToXlsxConverter {
    manifest: ConverterManifest,
}

impl PdfToXlsxConverter {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "pdf_to_xlsx".to_string(),
                name: "PDF to XLSX Converter".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT".to_string(),
                input_formats: vec!["pdf".into()],
                output_formats: vec!["xlsx".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 80,
                capabilities: vec!["pdf_to_xlsx".into()],
            },
        }
    }
}

impl Converter for PdfToXlsxConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        from == "pdf" && to == "xlsx"
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let text = super::pdf_extract::extract_pdf_text(&request.input_path)?;

        let stem = request
            .input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = job_dir.join("output").join(format!("{}.xlsx", stem));
        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;

        let mut workbook = rust_xlsxwriter::Workbook::new();
        let worksheet = workbook.add_worksheet();

        for (row_idx, line) in text.lines().enumerate() {
            let cells: Vec<&str> = if line.contains('\t') {
                line.split('\t').collect()
            } else {
                vec![line]
            };
            for (col_idx, cell) in cells.iter().enumerate() {
                let _ = worksheet.write_string(row_idx as u32, col_idx as u16, cell.trim());
            }
        }

        workbook.save(&output_path)
            .map_err(|e| ConversionError::engine_failure("pdf_to_xlsx", &format!("XLSX write failed: {}", e)))?;

        Ok(output_path)
    }
}
