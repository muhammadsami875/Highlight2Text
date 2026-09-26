pub mod extension;
pub mod magic;
pub mod reconcile;

use calamine::Reader;
use crate::conversion::job::{DetectedFileInfo, FileMetadata};
use std::path::Path;

pub fn detect_format(path: &Path) -> Result<DetectedFileInfo, String> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("Cannot read file: {}", e))?;

    let magic_format = magic::detect_by_magic(path).unwrap_or_default();
    let ext_format = extension::detect_by_extension(&extension);

    let (detected_format, mime_type, format_mismatch) =
        reconcile::reconcile(&magic_format, &ext_format, &extension);

    let metadata = extract_metadata(path, &detected_format);

    Ok(DetectedFileInfo {
        path: path.to_string_lossy().to_string(),
        name: file_name,
        extension,
        detected_format,
        mime_type,
        size_bytes: file_size,
        format_mismatch,
        metadata,
    })
}

fn extract_metadata(path: &Path, format: &str) -> FileMetadata {
    let mut meta = FileMetadata::default();

    match format {
        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tiff" | "gif" => {
            if let Ok(reader) = image::ImageReader::open(path) {
                if let Ok(dims) = reader.into_dimensions() {
                    meta.image_width = Some(dims.0);
                    meta.image_height = Some(dims.1);
                }
            }
        }
        "pdf" => {
            if let Ok(doc) = lopdf::Document::load(path) {
                meta.page_count = Some(doc.get_pages().len() as u32);
                meta.has_text_layer = Some(true);
            }
        }
        "xlsx" | "xls" | "ods" => {
            if let Ok(workbook) = calamine::open_workbook_auto(path) {
                meta.sheet_count = Some(workbook.sheet_names().len() as u32);
            }
        }
        _ => {}
    }

    meta
}
