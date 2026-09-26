use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use std::path::PathBuf;

pub struct MarkdownToHtmlConverter {
    manifest: ConverterManifest,
}

impl MarkdownToHtmlConverter {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "markdown_to_html".to_string(),
                name: "Markdown to HTML".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT".to_string(),
                input_formats: vec!["md".into()],
                output_formats: vec!["html".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 100,
                capabilities: vec!["markdown_conversion".into()],
            },
        }
    }
}

impl Converter for MarkdownToHtmlConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        from == "md" && to == "html"
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let markdown = std::fs::read_to_string(&request.input_path)
            .map_err(|e| ConversionError::io_error(&format!("Cannot read input: {}", e)))?;

        let parser = pulldown_cmark::Parser::new(&markdown);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);

        let full_html = format!(
            "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n<title>{}</title>\n\
             <style>body{{font-family:sans-serif;max-width:800px;margin:0 auto;padding:20px;line-height:1.6}}\
             pre{{background:#f4f4f4;padding:16px;overflow-x:auto;border-radius:4px}}\
             code{{background:#f4f4f4;padding:2px 4px;border-radius:2px}}\
             img{{max-width:100%}}</style>\n</head>\n<body>\n{}\n</body>\n</html>",
            request.input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("Document"),
            html_output
        );

        let stem = request.input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let output_path = job_dir.join("output").join(format!("{}.html", stem));
        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;

        std::fs::write(&output_path, full_html)
            .map_err(|e| ConversionError::io_error(&format!("Cannot write output: {}", e)))?;

        Ok(output_path)
    }
}
