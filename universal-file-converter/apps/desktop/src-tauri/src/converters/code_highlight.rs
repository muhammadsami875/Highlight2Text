use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use std::path::PathBuf;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

pub struct CodeHighlightConverter {
    manifest: ConverterManifest,
}

impl CodeHighlightConverter {
    pub fn new() -> Self {
        let code_formats: Vec<String> = vec![
            "py", "js", "ts", "java", "cpp", "c", "cs", "php",
            "rb", "go", "rs", "css", "sql", "sh", "yaml",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        Self {
            manifest: ConverterManifest {
                id: "code_highlight".to_string(),
                name: "Code Syntax Highlighter".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT".to_string(),
                input_formats: code_formats,
                output_formats: vec!["html".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 90,
                capabilities: vec!["syntax_highlighting".into()],
            },
        }
    }

    fn format_to_syntax_name(format: &str) -> &str {
        match format {
            "py" => "Python",
            "js" => "JavaScript",
            "ts" => "TypeScript",
            "java" => "Java",
            "cpp" | "c" => "C++",
            "cs" => "C#",
            "php" => "PHP",
            "rb" => "Ruby",
            "go" => "Go",
            "rs" => "Rust",
            "css" => "CSS",
            "sql" => "SQL",
            "sh" => "Bourne Again Shell (bash)",
            "yaml" => "YAML",
            _ => "Plain Text",
        }
    }
}

impl Converter for CodeHighlightConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        to == "html" && self.manifest.input_formats.contains(&from.to_string())
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let source = std::fs::read_to_string(&request.input_path)
            .map_err(|e| ConversionError::io_error(&format!("Cannot read input: {}", e)))?;

        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        let theme_name = request
            .options
            .syntax_theme
            .as_deref()
            .unwrap_or("InspiredGitHub");
        let theme = ts.themes.get(theme_name).unwrap_or_else(|| {
            ts.themes.values().next().unwrap()
        });

        let syntax_name = Self::format_to_syntax_name(&request.detected_format);
        let syntax = ss
            .find_syntax_by_name(syntax_name)
            .or_else(|| ss.find_syntax_by_extension(&request.detected_format))
            .unwrap_or_else(|| ss.find_syntax_plain_text());

        let highlighted = highlighted_html_for_string(&source, &ss, syntax, theme)
            .map_err(|e| ConversionError::engine_failure("code_highlight", &format!("Highlight error: {}", e)))?;

        let show_line_numbers = request.options.line_numbers.unwrap_or(true);
        let body = if show_line_numbers {
            let lines: Vec<&str> = highlighted.lines().collect();
            let mut numbered = String::from("<table style=\"border-collapse:collapse;width:100%\"><tbody>");
            let mut line_num = 1;
            for line in &lines {
                if line.contains("<pre") || line.contains("</pre>") {
                    continue;
                }
                numbered.push_str(&format!(
                    "<tr><td style=\"text-align:right;padding:0 12px 0 0;color:#999;user-select:none;white-space:nowrap\">{}</td><td style=\"padding:0\"><pre style=\"margin:0;display:inline\">{}</pre></td></tr>",
                    line_num, line
                ));
                line_num += 1;
            }
            numbered.push_str("</tbody></table>");
            numbered
        } else {
            highlighted
        };

        let title = request.input_path.file_name().and_then(|s| s.to_str()).unwrap_or("Code");
        let full_html = format!(
            "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n\
             <title>{}</title>\n\
             <style>body{{margin:0;padding:20px;font-family:monospace}}</style>\n\
             </head>\n<body>\n{}\n</body>\n</html>",
            title, body
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
