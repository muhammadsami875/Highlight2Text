use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use std::path::PathBuf;

pub struct TextToDocxConverter {
    manifest: ConverterManifest,
}

impl TextToDocxConverter {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "text_to_docx".to_string(),
                name: "Text/Markdown to DOCX Converter".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT".to_string(),
                input_formats: vec!["txt".into(), "md".into()],
                output_formats: vec!["docx".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 90,
                capabilities: vec!["text_to_docx".into()],
            },
        }
    }
}

impl Converter for TextToDocxConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        (from == "txt" || from == "md") && to == "docx"
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let content = std::fs::read_to_string(&request.input_path)
            .map_err(|e| ConversionError::io_error(&format!("Cannot read input: {}", e)))?;

        let mut docx = docx_rs::Docx::new();

        if request.detected_format == "md" {
            let parser = pulldown_cmark::Parser::new(&content);
            let mut current_text = String::new();
            let mut is_heading = false;
            let mut heading_level: u32 = 0;
            let mut is_bold = false;
            let mut is_italic = false;
            let mut current_runs: Vec<docx_rs::Run> = Vec::new();

            for event in parser {
                match event {
                    pulldown_cmark::Event::Start(tag) => {
                        match &tag {
                            pulldown_cmark::Tag::Heading { level, .. } => {
                                is_heading = true;
                                heading_level = *level as u32;
                            }
                            pulldown_cmark::Tag::Strong => is_bold = true,
                            pulldown_cmark::Tag::Emphasis => is_italic = true,
                            _ => {}
                        }
                    }
                    pulldown_cmark::Event::End(tag_end) => {
                        match tag_end {
                            pulldown_cmark::TagEnd::Paragraph
                            | pulldown_cmark::TagEnd::Heading(_) => {
                                if !current_text.is_empty() {
                                    let mut run = docx_rs::Run::new().add_text(&current_text);
                                    if is_bold {
                                        run = run.bold();
                                    }
                                    if is_italic {
                                        run = run.italic();
                                    }
                                    current_runs.push(run);
                                    current_text.clear();
                                }

                                let mut para = docx_rs::Paragraph::new();
                                if is_heading {
                                    let size = match heading_level {
                                        1 => 32,
                                        2 => 26,
                                        3 => 22,
                                        _ => 20,
                                    };
                                    for r in current_runs.drain(..) {
                                        para = para.add_run(r.bold().size(size * 2));
                                    }
                                    is_heading = false;
                                } else {
                                    for r in current_runs.drain(..) {
                                        para = para.add_run(r);
                                    }
                                }
                                docx = docx.add_paragraph(para);
                            }
                            pulldown_cmark::TagEnd::Strong => {
                                if !current_text.is_empty() {
                                    current_runs.push(
                                        docx_rs::Run::new().add_text(&current_text).bold(),
                                    );
                                    current_text.clear();
                                }
                                is_bold = false;
                            }
                            pulldown_cmark::TagEnd::Emphasis => {
                                if !current_text.is_empty() {
                                    current_runs.push(
                                        docx_rs::Run::new().add_text(&current_text).italic(),
                                    );
                                    current_text.clear();
                                }
                                is_italic = false;
                            }
                            pulldown_cmark::TagEnd::Item => {
                                if !current_text.is_empty() {
                                    let run = docx_rs::Run::new()
                                        .add_text(&format!("  - {}", &current_text));
                                    let para = docx_rs::Paragraph::new().add_run(run);
                                    docx = docx.add_paragraph(para);
                                    current_text.clear();
                                }
                                current_runs.clear();
                            }
                            _ => {}
                        }
                    }
                    pulldown_cmark::Event::Text(text) => {
                        current_text.push_str(&text);
                    }
                    pulldown_cmark::Event::Code(code) => {
                        if !current_text.is_empty() {
                            let mut run = docx_rs::Run::new().add_text(&current_text);
                            if is_bold { run = run.bold(); }
                            if is_italic { run = run.italic(); }
                            current_runs.push(run);
                            current_text.clear();
                        }
                        current_runs.push(
                            docx_rs::Run::new()
                                .add_text(&format!("`{}`", &code))
                                .fonts(docx_rs::RunFonts::new().ascii("Courier New")),
                        );
                    }
                    pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                        current_text.push(' ');
                    }
                    _ => {}
                }
            }

            if !current_text.is_empty() || !current_runs.is_empty() {
                if !current_text.is_empty() {
                    current_runs.push(docx_rs::Run::new().add_text(&current_text));
                }
                let mut para = docx_rs::Paragraph::new();
                for r in current_runs {
                    para = para.add_run(r);
                }
                docx = docx.add_paragraph(para);
            }
        } else {
            for line in content.lines() {
                let para = docx_rs::Paragraph::new()
                    .add_run(docx_rs::Run::new().add_text(line));
                docx = docx.add_paragraph(para);
            }
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
            .map_err(|e| ConversionError::engine_failure("text_to_docx", &format!("DOCX write failed: {}", e)))?;

        Ok(output_path)
    }
}
