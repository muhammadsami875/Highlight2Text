use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use calamine::{open_workbook_auto, Data, Reader};
use std::path::PathBuf;

pub struct XlsxExtractConverter {
    manifest: ConverterManifest,
}

impl XlsxExtractConverter {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "xlsx_extract".to_string(),
                name: "Spreadsheet Data Extractor".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT".to_string(),
                input_formats: vec!["xlsx".into(), "xls".into(), "ods".into()],
                output_formats: vec!["csv".into(), "tsv".into(), "json".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 100,
                capabilities: vec!["data_extraction".into()],
            },
        }
    }

    fn extract_to_csv(path: &std::path::Path) -> Result<String, ConversionError> {
        let mut workbook = open_workbook_auto(path)
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot open spreadsheet: {}", e)))?;

        let sheet_names = workbook.sheet_names().to_vec();
        if sheet_names.is_empty() {
            return Err(ConversionError::corrupt_input("No sheets found"));
        }

        let range = workbook
            .worksheet_range(&sheet_names[0])
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot read sheet: {}", e)))?;

        let mut wtr = csv::Writer::from_writer(Vec::new());

        for row in range.rows() {
            let fields: Vec<String> = row.iter().map(cell_to_string).collect();
            wtr.write_record(&fields)
                .map_err(|e| ConversionError::engine_failure("xlsx_extract", &format!("CSV write error: {}", e)))?;
        }

        let bytes = wtr.into_inner()
            .map_err(|e| ConversionError::engine_failure("xlsx_extract", &format!("CSV flush error: {}", e)))?;
        String::from_utf8(bytes)
            .map_err(|e| ConversionError::engine_failure("xlsx_extract", &format!("UTF-8 error: {}", e)))
    }

    fn extract_to_json(path: &std::path::Path) -> Result<String, ConversionError> {
        let mut workbook = open_workbook_auto(path)
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot open spreadsheet: {}", e)))?;

        let sheet_names = workbook.sheet_names().to_vec();
        if sheet_names.is_empty() {
            return Err(ConversionError::corrupt_input("No sheets found"));
        }

        let range = workbook
            .worksheet_range(&sheet_names[0])
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot read sheet: {}", e)))?;

        let mut rows_iter = range.rows();
        let headers: Vec<String> = match rows_iter.next() {
            Some(row) => row.iter().map(cell_to_string).collect(),
            None => return Ok("[]".to_string()),
        };

        let mut records = Vec::new();
        for row in rows_iter {
            let mut obj = serde_json::Map::new();
            for (i, cell) in row.iter().enumerate() {
                let key = headers.get(i).cloned().unwrap_or_else(|| format!("col_{}", i));
                let value = match cell {
                    Data::Empty => serde_json::Value::Null,
                    Data::Float(f) => serde_json::json!(*f),
                    Data::Int(i) => serde_json::json!(*i),
                    Data::Bool(b) => serde_json::json!(*b),
                    _ => serde_json::Value::String(cell_to_string(cell)),
                };
                obj.insert(key, value);
            }
            records.push(serde_json::Value::Object(obj));
        }

        serde_json::to_string_pretty(&records)
            .map_err(|e| ConversionError::engine_failure("xlsx_extract", &format!("JSON error: {}", e)))
    }

    fn extract_to_tsv(path: &std::path::Path) -> Result<String, ConversionError> {
        let mut workbook = open_workbook_auto(path)
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot open spreadsheet: {}", e)))?;

        let sheet_names = workbook.sheet_names().to_vec();
        if sheet_names.is_empty() {
            return Err(ConversionError::corrupt_input("No sheets found"));
        }

        let range = workbook
            .worksheet_range(&sheet_names[0])
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot read sheet: {}", e)))?;

        let mut output = String::new();
        for row in range.rows() {
            let fields: Vec<String> = row.iter().map(cell_to_string).collect();
            output.push_str(&fields.join("\t"));
            output.push('\n');
        }
        Ok(output)
    }
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if *f == (*f as i64) as f64 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => format!("{}", b),
        Data::Error(e) => format!("{:?}", e),
        Data::DateTime(dt) => format!("{}", dt.as_f64()),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}

impl Converter for XlsxExtractConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        if from == to {
            return false;
        }
        matches!(from, "xlsx" | "xls" | "ods") && matches!(to, "csv" | "tsv" | "json")
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let stem = request
            .input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let (content, ext) = match request.output_format.as_str() {
            "csv" => (Self::extract_to_csv(&request.input_path)?, "csv"),
            "tsv" => (Self::extract_to_tsv(&request.input_path)?, "tsv"),
            "json" => (Self::extract_to_json(&request.input_path)?, "json"),
            _ => {
                return Err(ConversionError::unsupported(
                    &request.detected_format,
                    &request.output_format,
                ))
            }
        };

        let output_path = job_dir.join("output").join(format!("{}.{}", stem, ext));
        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;

        std::fs::write(&output_path, content)
            .map_err(|e| ConversionError::io_error(&format!("Cannot write output: {}", e)))?;

        Ok(output_path)
    }
}
