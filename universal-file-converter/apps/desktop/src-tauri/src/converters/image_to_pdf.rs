use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use std::path::PathBuf;

pub struct ImageToPdfConverter {
    manifest: ConverterManifest,
}

impl ImageToPdfConverter {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "image_to_pdf".to_string(),
                name: "Image to PDF".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT".to_string(),
                input_formats: vec![
                    "png".into(), "jpg".into(), "webp".into(),
                    "bmp".into(), "tiff".into(),
                ],
                output_formats: vec!["pdf".into()],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 100,
                capabilities: vec!["image_to_pdf".into()],
            },
        }
    }
}

impl Converter for ImageToPdfConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        to == "pdf" && self.manifest.input_formats.contains(&from.to_string())
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let img = image::open(&request.input_path)
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot decode image: {}", e)))?;

        let (width, height) = (img.width() as f32, img.height() as f32);
        let dpi = request.options.dpi.unwrap_or(150) as f32;

        let page_width_pt = width / dpi * 72.0;
        let page_height_pt = height / dpi * 72.0;

        let stem = request
            .input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = job_dir.join("output").join(format!("{}.pdf", stem));
        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;

        let mut doc = lopdf::Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();
        let content_id = doc.new_object_id();

        let content = lopdf::content::Content {
            operations: vec![],
        };
        let content_data = content.encode().unwrap_or_default();

        let mut stream_dict = lopdf::Dictionary::new();
        stream_dict.set("Length", lopdf::Object::Integer(content_data.len() as i64));

        doc.objects.insert(
            content_id,
            lopdf::Object::Stream(lopdf::Stream::new(stream_dict, content_data)),
        );

        let media_box = lopdf::Object::Array(vec![
            lopdf::Object::Integer(0),
            lopdf::Object::Integer(0),
            lopdf::Object::Real(page_width_pt),
            lopdf::Object::Real(page_height_pt),
        ]);

        let mut page_dict = lopdf::Dictionary::new();
        page_dict.set("Type", lopdf::Object::Name(b"Page".to_vec()));
        page_dict.set("Parent", lopdf::Object::Reference(pages_id));
        page_dict.set("MediaBox", media_box);
        page_dict.set("Contents", lopdf::Object::Reference(content_id));
        doc.objects.insert(page_id, lopdf::Object::Dictionary(page_dict));

        let mut pages_dict = lopdf::Dictionary::new();
        pages_dict.set("Type", lopdf::Object::Name(b"Pages".to_vec()));
        pages_dict.set("Kids", lopdf::Object::Array(vec![lopdf::Object::Reference(page_id)]));
        pages_dict.set("Count", lopdf::Object::Integer(1));
        doc.objects.insert(pages_id, lopdf::Object::Dictionary(pages_dict));

        let catalog_id = doc.new_object_id();
        let mut catalog = lopdf::Dictionary::new();
        catalog.set("Type", lopdf::Object::Name(b"Catalog".to_vec()));
        catalog.set("Pages", lopdf::Object::Reference(pages_id));
        doc.objects.insert(catalog_id, lopdf::Object::Dictionary(catalog));
        doc.trailer.set("Root", lopdf::Object::Reference(catalog_id));

        doc.save(&output_path)
            .map_err(|e| ConversionError::engine_failure("image_to_pdf", &format!("PDF save failed: {}", e)))?;

        Ok(output_path)
    }
}
