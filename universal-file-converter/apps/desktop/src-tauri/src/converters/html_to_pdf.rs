use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use lopdf::{Document, Object, Stream};
use std::path::PathBuf;

pub struct HtmlToPdfConverter {
    manifest: ConverterManifest,
}

impl HtmlToPdfConverter {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "html_to_pdf".to_string(),
                name: "HTML to PDF Converter".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT".to_string(),
                input_formats: vec!["html".into()],
                output_formats: vec!["pdf".into(), "txt".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 70,
                capabilities: vec!["html_to_pdf".into()],
            },
        }
    }
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut tag_name = String::new();
    let mut collecting_tag = false;

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            collecting_tag = true;
            tag_name.clear();
            continue;
        }
        if ch == '>' {
            in_tag = false;
            collecting_tag = false;
            let lower = tag_name.to_lowercase();
            if lower == "script" {
                in_script = true;
            } else if lower == "/script" {
                in_script = false;
            } else if lower == "style" {
                in_style = true;
            } else if lower == "/style" {
                in_style = false;
            } else if lower == "br" || lower == "br/" || lower == "br /" {
                result.push('\n');
            } else if lower == "p"
                || lower == "/p"
                || lower == "div"
                || lower == "/div"
                || lower == "h1"
                || lower == "h2"
                || lower == "h3"
                || lower == "h4"
                || lower == "h5"
                || lower == "h6"
                || lower == "tr"
                || lower == "/tr"
                || lower == "li"
            {
                result.push('\n');
            } else if lower == "td" || lower == "th" {
                result.push('\t');
            }
            continue;
        }
        if in_tag {
            if collecting_tag && (ch.is_alphanumeric() || ch == '/') {
                tag_name.push(ch);
            } else {
                collecting_tag = false;
            }
            continue;
        }
        if in_script || in_style {
            continue;
        }
        result.push(ch);
    }

    decode_html_entities(&result)
}

fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
}

fn pdf_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

impl Converter for HtmlToPdfConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        from == "html" && (to == "pdf" || to == "txt")
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let html = std::fs::read_to_string(&request.input_path)
            .map_err(|e| ConversionError::io_error(&format!("Cannot read HTML: {}", e)))?;

        let text = strip_html_tags(&html);

        if request.output_format == "txt" {
            let stem = request
                .input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let output_path = job_dir.join("output").join(format!("{}.txt", stem));
            std::fs::create_dir_all(output_path.parent().unwrap())
                .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;
            std::fs::write(&output_path, text.trim())
                .map_err(|e| ConversionError::io_error(&format!("Cannot write output: {}", e)))?;
            return Ok(output_path);
        }

        let font_size: f32 = 10.0;
        let line_height = font_size * 1.4;
        let margin: f32 = 72.0;
        let page_width: f32 = 612.0;
        let page_height: f32 = 792.0;
        let usable_width = page_width - 2.0 * margin;
        let usable_height = page_height - 2.0 * margin;
        let lines_per_page = (usable_height / line_height) as usize;
        let char_width = font_size * 0.6;
        let max_chars = (usable_width / char_width) as usize;

        let mut wrapped: Vec<String> = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                wrapped.push(String::new());
                continue;
            }
            if trimmed.len() <= max_chars {
                wrapped.push(trimmed.to_string());
            } else {
                let mut rem = trimmed;
                while rem.len() > max_chars {
                    let (chunk, rest) = rem.split_at(max_chars);
                    wrapped.push(chunk.to_string());
                    rem = rest;
                }
                if !rem.is_empty() {
                    wrapped.push(rem.to_string());
                }
            }
        }

        let mut doc = Document::with_version("1.7");

        let mut font_dict = lopdf::Dictionary::new();
        font_dict.set("Type", Object::Name(b"Font".to_vec()));
        font_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
        font_dict.set("BaseFont", Object::Name(b"Helvetica".to_vec()));
        let font_obj_id = doc.add_object(font_dict);

        let mut pages_dict = lopdf::Dictionary::new();
        pages_dict.set("Type", Object::Name(b"Pages".to_vec()));
        pages_dict.set("Kids", Object::Array(Vec::new()));
        pages_dict.set("Count", Object::Integer(0));
        let pages_id = doc.add_object(pages_dict);

        let chunks: Vec<&[String]> = wrapped.chunks(lines_per_page).collect();
        let total_pages = chunks.len().max(1);
        let mut page_ids = Vec::new();

        for chunk in &chunks {
            let mut content = String::new();
            content.push_str("BT\n");
            content.push_str(&format!("/F1 {} Tf\n", font_size));

            let start_y = page_height - margin;
            for (i, line) in chunk.iter().enumerate() {
                let y = start_y - (i as f32 * line_height);
                content.push_str(&format!(
                    "{} {} Td ({}) Tj\n",
                    margin,
                    y,
                    pdf_escape(line)
                ));
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

            let mut res_fonts = lopdf::Dictionary::new();
            res_fonts.set("F1", Object::Reference(font_obj_id));
            let mut resources = lopdf::Dictionary::new();
            resources.set("Font", Object::Dictionary(res_fonts));

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
            .map_err(|e| ConversionError::engine_failure("html_to_pdf", &format!("PDF error: {}", e)))?;

        let stem = request
            .input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = job_dir.join("output").join(format!("{}.pdf", stem));
        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;
        std::fs::write(&output_path, buf)
            .map_err(|e| ConversionError::io_error(&format!("Cannot write output: {}", e)))?;

        Ok(output_path)
    }
}
