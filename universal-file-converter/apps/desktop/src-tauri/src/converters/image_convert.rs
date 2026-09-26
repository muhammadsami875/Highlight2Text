use crate::conversion::error::ConversionError;
use crate::conversion::request::ConversionRequest;
use crate::engines::manifest::{ConverterManifest, EngineType, Platform};
use crate::engines::registry::Converter;
use image::ImageFormat;
use std::path::PathBuf;

pub struct ImageConverter {
    manifest: ConverterManifest,
}

impl ImageConverter {
    pub fn new() -> Self {
        Self {
            manifest: ConverterManifest {
                id: "image_convert".to_string(),
                name: "Image Converter".to_string(),
                version: "0.1.0".to_string(),
                engine_type: EngineType::Bundled,
                license: "MIT / Apache-2.0".to_string(),
                input_formats: vec![
                    "png".into(), "jpg".into(), "webp".into(),
                    "bmp".into(), "tiff".into(), "gif".into(),
                    "ico".into(),
                ],
                output_formats: vec![
                    "png".into(), "jpg".into(), "webp".into(),
                    "bmp".into(), "tiff".into(), "gif".into(),
                    "ico".into(),
                ],
                platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
                priority: 100,
                capabilities: vec!["image_conversion".into()],
            },
        }
    }
}

impl Converter for ImageConverter {
    fn manifest(&self) -> &ConverterManifest {
        &self.manifest
    }

    fn is_available(&self) -> bool {
        true
    }

    fn can_convert(&self, from: &str, to: &str) -> bool {
        from != to
            && self.manifest.input_formats.contains(&from.to_string())
            && self.manifest.output_formats.contains(&to.to_string())
    }

    fn convert(
        &self,
        request: &ConversionRequest,
        job_dir: &PathBuf,
    ) -> Result<PathBuf, ConversionError> {
        let img = image::open(&request.input_path)
            .map_err(|e| ConversionError::corrupt_input(&format!("Cannot decode image: {}", e)))?;

        let output_format = match request.output_format.as_str() {
            "png" => ImageFormat::Png,
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            "bmp" => ImageFormat::Bmp,
            "tiff" => ImageFormat::Tiff,
            "webp" => ImageFormat::WebP,
            "gif" => ImageFormat::Gif,
            "ico" => ImageFormat::Ico,
            other => {
                return Err(ConversionError::unsupported(
                    &request.detected_format,
                    other,
                ))
            }
        };

        let stem = request
            .input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let ext = match request.output_format.as_str() {
            "jpg" => "jpg",
            other => other,
        };
        let output_path = job_dir.join("output").join(format!("{}.{}", stem, ext));
        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| ConversionError::io_error(&format!("Cannot create output dir: {}", e)))?;

        img.save_with_format(&output_path, output_format)
            .map_err(|e| {
                ConversionError::engine_failure("image_convert", &format!("Save failed: {}", e))
            })?;

        Ok(output_path)
    }
}
