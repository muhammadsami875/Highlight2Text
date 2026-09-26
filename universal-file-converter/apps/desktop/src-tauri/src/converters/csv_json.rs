use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use std::path::PathBuf;

pub struct CsvJsonConverter {
    manifest: ConverterManifest,
}

impl CsvJsonConverter {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "csv_json".to_string(),
                name: "CSV/JSON Converter".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT".to_string(),
                input_formats: vec!["csv".into(), "json".into()],
                output_formats: vec!["csv".into(), "json".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 100,
                capabilities: vec!["data_conversion".into()],
            },
        }
    }

    fn csv_to_json(input: &std::path::Path) -> Result<String, ConversionError> {
        let mut reader = csv::Reader::from_path(input)
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot read CSV: {}", e)))?;

        let headers: Vec<String> = reader
            .headers()
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot read CSV headers: {}", e)))?
            .iter()
            .map(|h| h.to_string())
            .collect();

        let mut records = Vec::new();
        for result in reader.records() {
            let record = result
                .map_err(|e| ConversionError::corrupt_input(&format!("CSV parse error: {}", e)))?;
            let mut obj = serde_json::Map::new();
            for (i, field) in record.iter().enumerate() {
                let key = headers.get(i).cloned().unwrap_or_else(|| format!("col_{}", i));
                obj.insert(key, serde_json::Value::String(field.to_string()));
            }
            records.push(serde_json::Value::Object(obj));
        }

        serde_json::to_string_pretty(&records)
            .map_err(|e| ConversionError::engine_failure("csv_json", &format!("JSON serialize error: {}", e)))
    }

    fn json_to_csv(input: &std::path::Path) -> Result<String, ConversionError> {
        let data = std::fs::read_to_string(input)
            .map_err(|e| ConversionError::io_error(&format!("Cannot read JSON: {}", e)))?;

        let value: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| ConversionError::corrupt_input(&format!("Invalid JSON: {}", e)))?;

        let array = match &value {
            serde_json::Value::Array(arr) => arr,
            _ => return Err(ConversionError::corrupt_input("JSON must be an array of objects")),
        };

        if array.is_empty() {
            return Ok(String::new());
        }

        let headers: Vec<String> = match &array[0] {
            serde_json::Value::Object(obj) => obj.keys().cloned().collect(),
            _ => return Err(ConversionError::corrupt_input("JSON array must contain objects")),
        };

        let mut wtr = csv::Writer::from_writer(Vec::new());
        wtr.write_record(&headers)
            .map_err(|e| ConversionError::engine_failure("csv_json", &format!("CSV write error: {}", e)))?;

        for item in array {
            if let serde_json::Value::Object(obj) = item {
                let row: Vec<String> = headers
                    .iter()
                    .map(|h| match obj.get(h) {
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(v) => v.to_string(),
                        None => String::new(),
                    })
                    .collect();
                wtr.write_record(&row)
                    .map_err(|e| ConversionError::engine_failure("csv_json", &format!("CSV write error: {}", e)))?;
            }
        }

        let bytes = wtr.into_inner()
            .map_err(|e| ConversionError::engine_failure("csv_json", &format!("CSV flush error: {}", e)))?;
        String::from_utf8(bytes)
            .map_err(|e| ConversionError::engine_failure("csv_json", &format!("UTF-8 error: {}", e)))
    }
}

impl Converter for CsvJsonConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        (from == "csv" && to == "json") || (from == "json" && to == "csv")
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let stem = request.input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let (content, ext) = match (request.detected_format.as_str(), request.output_format.as_str()) {
            ("csv", "json") => (Self::csv_to_json(&request.input_path)?, "json"),
            ("json", "csv") => (Self::json_to_csv(&request.input_path)?, "csv"),
            _ => return Err(ConversionError::unsupported(&request.detected_format, &request.output_format)),
        };

        let output_path = job_dir.join("output").join(format!("{}.{}", stem, ext));
        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;

        std::fs::write(&output_path, content)
            .map_err(|e| ConversionError::io_error(&format!("Cannot write output: {}", e)))?;

        Ok(output_path)
    }
}
