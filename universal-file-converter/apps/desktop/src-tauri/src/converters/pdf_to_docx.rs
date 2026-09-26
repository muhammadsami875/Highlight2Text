use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use std::path::PathBuf;

pub struct PdfToDocxConverter {
    manifest: ConverterManifest,
}

impl PdfToDocxConverter {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "pdf_to_docx".to_string(),
                name: "PDF to DOCX Converter".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT".to_string(),
                input_formats: vec!["pdf".into()],
                output_formats: vec!["docx".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 80,
                capabilities: vec!["pdf_to_docx".into()],
            },
        }
    }
}

impl Converter for PdfToDocxConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        from == "pdf" && to == "docx"
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let text = super::pdf_extract::extract_pdf_text(&request.input_path)?;

        let mut docx = docx_rs::Docx::new();
        for line in text.lines() {
            let para = docx_rs::Paragraph::new()
                .add_run(docx_rs::Run::new().add_text(line));
            docx = docx.add_paragraph(para);
        }

        let stem = request
            .input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = job_dir.join("output").join(format!("{}.docx", stem));
        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;

        let file = std::fs::File::create(&output_path)
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output: {}", e)))?;
        let mut buf = std::io::BufWriter::new(file);
        docx.build()
            .pack(&mut buf)
            .map_err(|e| ConversionError::engine_failure("pdf_to_docx", &format!("DOCX write failed: {}", e)))?;

        Ok(output_path)
    }
}
