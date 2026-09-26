use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use lopdf::{Document, Object, Stream};
use std::path::PathBuf;

const INPUT_FORMATS: &[&str] = &[
    "txt", "md", "py", "rs", "js", "ts", "java", "c", "cpp", "go", "rb", "php", "swift", "kt",
    "sh", "css", "sql", "yaml", "yml", "toml", "ini", "xml", "log", "conf",
];

pub struct TextToPdfConverter {
    manifest: ConverterManifest,
}

impl TextToPdfConverter {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "text_to_pdf".to_string(),
                name: "Text to PDF Converter".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT".to_string(),
                input_formats: INPUT_FORMATS.iter().map(|s| s.to_string()).collect(),
                output_formats: vec!["pdf".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 80,
                capabilities: vec!["text_to_pdf".into()],
            },
        }
    }

    fn render_text_to_pdf(
        text: &str,
        request: &ConversionRequest,
    ) -> Result<Vec<u8>, ConversionError> {
        let font_size = request.options.font_size.unwrap_or(10) as f32;
        let line_height = font_size * 1.4;
        let margin = 72.0_f32;
        let page_width = 612.0_f32;
        let page_height = 792.0_f32;
        let usable_width = page_width - 2.0 * margin;
        let usable_height = page_height - 2.0 * margin;
        let lines_per_page = (usable_height / line_height) as usize;
        let show_line_numbers = request.options.line_numbers.unwrap_or(false);

        let char_width = font_size * 0.6;
        let max_chars = (usable_width / char_width) as usize;

        let mut wrapped_lines: Vec<String> = Vec::new();
        for line in text.lines() {
            if line.len() <= max_chars {
                wrapped_lines.push(line.to_string());
            } else {
                let mut remaining = line;
                while remaining.len() > max_chars {
                    let (chunk, rest) = remaining.split_at(max_chars);
                    wrapped_lines.push(chunk.to_string());
                    remaining = rest;
                }
                if !remaining.is_empty() {
                    wrapped_lines.push(remaining.to_string());
                }
            }
        }

        let mut doc = Document::with_version("1.7");

        let font_id = doc.add_object(Stream::new(
            lopdf::Dictionary::new(),
            Vec::new(),
        ));
        let _ = font_id;

        let mut font_dict = lopdf::Dictionary::new();
        font_dict.set("Type", Object::Name(b"Font".to_vec()));
        font_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
        font_dict.set("BaseFont", Object::Name(b"Courier".to_vec()));
        let font_obj_id = doc.add_object(font_dict);

        let mut pages_dict = lopdf::Dictionary::new();
        pages_dict.set("Type", Object::Name(b"Pages".to_vec()));
        pages_dict.set("Kids", Object::Array(Vec::new()));
        pages_dict.set("Count", Object::Integer(0));
        let pages_id = doc.add_object(pages_dict);

        let chunks: Vec<&[String]> = wrapped_lines.chunks(lines_per_page).collect();
        let total_pages = chunks.len().max(1);
        let mut page_ids = Vec::new();

        for (page_idx, chunk) in chunks.iter().enumerate() {
            let mut content = String::new();
            content.push_str("BT\n");
            content.push_str(&format!("/F1 {} Tf\n", font_size));

            let start_y = page_height - margin;
            for (i, line) in chunk.iter().enumerate() {
                let y = start_y - (i as f32 * line_height);
                let escaped = pdf_escape_string(line);

                if show_line_numbers {
                    let line_num = page_idx * lines_per_page + i + 1;
                    content.push_str(&format!(
                        "{} {} Td ({:>4} {}) Tj\n",
                        margin, y, line_num, escaped
                    ));
                } else {
                    content.push_str(&format!(
                        "{} {} Td ({}) Tj\n",
                        margin, y, escaped
                    ));
                }
            }
            content.push_str("ET\n");

            let content_stream = Stream::new(lopdf::Dictionary::new(), content.into_bytes());
            let content_id = doc.add_object(content_stream);

            let media_box = Object::Array(vec![
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(page_width.into()),
                Object::Real(page_height.into()),
            ]);

            let mut resources_fonts = lopdf::Dictionary::new();
            resources_fonts.set("F1", Object::Reference(font_obj_id));
            let mut resources = lopdf::Dictionary::new();
            resources.set("Font", Object::Dictionary(resources_fonts));

            let mut page_dict = lopdf::Dictionary::new();
            page_dict.set("Type", Object::Name(b"Page".to_vec()));
            page_dict.set("Parent", Object::Reference(pages_id));
            page_dict.set("MediaBox", media_box);
            page_dict.set("Contents", Object::Reference(content_id));
            page_dict.set("Resources", Object::Dictionary(resources));

            let page_id = doc.add_object(page_dict);
            page_ids.push(page_id);
        }

        let kids: Vec<Object> = page_ids.iter().map(|id| Object::Reference(*id)).collect();
        let pages_obj = doc.get_object_mut(pages_id).unwrap();
        if let Object::Dictionary(ref mut dict) = *pages_obj {
            dict.set("Kids", Object::Array(kids));
            dict.set("Count", Object::Integer(total_pages as i64));
        }

        let mut catalog = lopdf::Dictionary::new();
        catalog.set("Type", Object::Name(b"Catalog".to_vec()));
        catalog.set("Pages", Object::Reference(pages_id));
        let catalog_id = doc.add_object(catalog);

        doc.trailer.set("Root", Object::Reference(catalog_id));

        let mut buf = Vec::new();
        doc.save_to(&mut buf)
            .map_err(|e| ConversionError::engine_failure("text_to_pdf", &format!("PDF write error: {}", e)))?;

        Ok(buf)
    }
}

fn pdf_escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

impl Converter for TextToPdfConverter {
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
        INPUT_FORMATS.contains(&from) && to == "pdf"
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let text = std::fs::read_to_string(&request.input_path)
            .map_err(|e| ConversionError::io_error(&format!("Cannot read input: {}", e)))?;

        let pdf_bytes = Self::render_text_to_pdf(&text, request)?;

        let stem = request
            .input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = job_dir.join("output").join(format!("{}.pdf", stem));

        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;

        std::fs::write(&output_path, pdf_bytes)
            .map_err(|e| ConversionError::io_error(&format!("Cannot write output: {}", e)))?;

        Ok(output_path)
    }
}
