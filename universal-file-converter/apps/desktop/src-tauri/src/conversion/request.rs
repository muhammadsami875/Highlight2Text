use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionOptions {
    pub page_range: Option<String>,
    pub dpi: Option<u32>,
    pub image_quality: Option<u32>,
    pub compression: Option<String>,
    pub page_size: Option<String>,
    pub orientation: Option<String>,
    pub margins: Option<Margins>,
    pub font_size: Option<u32>,
    pub font_family: Option<String>,
    pub line_numbers: Option<bool>,
    pub line_wrapping: Option<bool>,
    pub syntax_theme: Option<String>,
    pub sheet_selection: Option<Vec<String>>,
    pub fit_to_page: Option<bool>,
    pub ocr_enabled: Option<bool>,
    pub ocr_language: Option<String>,
    pub preserve_layout: Option<bool>,
    pub background_color: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

#[derive(Debug, Clone)]
pub struct ConversionRequest {
    pub input_path: PathBuf,
    pub detected_format: String,
    pub output_format: String,
    pub output_dir: PathBuf,
    pub options: ConversionOptions,
}
