use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use std::path::PathBuf;

pub struct CsvXlsxConverter {
    manifest: ConverterManifest,
}

impl CsvXlsxConverter {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "csv_xlsx".to_string(),
                name: "CSV to XLSX".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT / Apache-2.0".to_string(),
                input_formats: vec!["csv".into()],
                output_formats: vec!["xlsx".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 100,
                capabilities: vec!["spreadsheet_conversion".into()],
            },
        }
    }
}

impl Converter for CsvXlsxConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        from == "csv" && to == "xlsx"
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let mut reader = csv::Reader::from_path(&request.input_path)
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot read CSV: {}", e)))?;

        let mut workbook = rust_xlsxwriter::Workbook::new();
        let worksheet = workbook.add_worksheet();

        let bold = rust_xlsxwriter::Format::new().set_bold();

        let headers = reader
            .headers()
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot read CSV headers: {}", e)))?
            .clone();

        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(0, col as u16, header, &bold)
                .map_err(|e| ConversionError::engine_failure("csv_xlsx", &format!("Write error: {}", e)))?;
        }

        for (row_idx, result) in reader.records().enumerate() {
            let record = result
                .map_err(|e| ConversionError::corrupt_input(&format!("CSV parse error: {}", e)))?;
            for (col, field) in record.iter().enumerate() {
                if let Ok(num) = field.parse::<f64>() {
                    worksheet
                        .write_number((row_idx + 1) as u32, col as u16, num)
                        .map_err(|e| ConversionError::engine_failure("csv_xlsx", &format!("Write error: {}", e)))?;
                } else {
                    worksheet
                        .write_string((row_idx + 1) as u32, col as u16, field)
                        .map_err(|e| ConversionError::engine_failure("csv_xlsx", &format!("Write error: {}", e)))?;
                }
            }
        }

        let stem = request.input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let output_path = job_dir.join("output").join(format!("{}.xlsx", stem));
        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;

        workbook
            .save(&output_path)
            .map_err(|e| ConversionError::engine_failure("csv_xlsx", &format!("XLSX save error: {}", e)))?;

        Ok(output_path)
    }
}
