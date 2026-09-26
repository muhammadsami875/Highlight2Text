use std::fs;
use std::io::Read;
use std::path::Path;

pub enum ValidationResult {
    Valid,
    ValidWithWarnings(Vec<String>),
    Invalid(String),
}

pub fn validate_output(path: &Path, format: &str) -> ValidationResult {
    if !path.exists() {
        return ValidationResult::Invalid("Output file does not exist".to_string());
    }

    let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if size == 0 {
        return ValidationResult::Invalid("Output file is empty".to_string());
    }

    match format {
        "pdf" => validate_pdf(path),
        "docx" | "xlsx" | "pptx" | "odt" | "ods" | "odp" => validate_openxml(path),
        "png" => validate_image(path, &[0x89, 0x50, 0x4E, 0x47]),
        "jpg" | "jpeg" => validate_image(path, &[0xFF, 0xD8, 0xFF]),
        "webp" => validate_webp(path),
        "bmp" => validate_image(path, &[0x42, 0x4D]),
        "tiff" | "tif" => validate_tiff(path),
        "txt" | "csv" | "tsv" | "html" | "md" | "json" | "xml" => validate_text(path),
        _ => ValidationResult::Valid,
    }
}

fn validate_pdf(path: &Path) -> ValidationResult {
    match lopdf::Document::load(path) {
        Ok(doc) => {
            let page_count = doc.get_pages().len();
            if page_count == 0 {
                ValidationResult::ValidWithWarnings(vec![
                    "PDF has 0 pages".to_string(),
                ])
            } else {
                ValidationResult::Valid
            }
        }
        Err(e) => ValidationResult::Invalid(format!("Invalid PDF: {}", e)),
    }
}

fn validate_openxml(path: &Path) -> ValidationResult {
    match zip::ZipArchive::new(fs::File::open(path).unwrap()) {
        Ok(archive) => {
            if archive.len() == 0 {
                ValidationResult::Invalid("Empty archive".to_string())
            } else {
                ValidationResult::Valid
            }
        }
        Err(e) => ValidationResult::Invalid(format!("Invalid Office document: {}", e)),
    }
}

fn validate_image(path: &Path, magic: &[u8]) -> ValidationResult {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return ValidationResult::Invalid(format!("Cannot open: {}", e)),
    };
    let mut header = vec![0u8; magic.len()];
    if file.read_exact(&mut header).is_err() {
        return ValidationResult::Invalid("File too small".to_string());
    }
    if header.starts_with(magic) {
        ValidationResult::Valid
    } else {
        ValidationResult::Invalid("Invalid image header".to_string())
    }
}

fn validate_webp(path: &Path) -> ValidationResult {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return ValidationResult::Invalid(format!("Cannot open: {}", e)),
    };
    let mut header = [0u8; 12];
    if file.read_exact(&mut header).is_err() {
        return ValidationResult::Invalid("File too small".to_string());
    }
    if &header[0..4] == b"RIFF" && &header[8..12] == b"WEBP" {
        ValidationResult::Valid
    } else {
        ValidationResult::Invalid("Invalid WebP header".to_string())
    }
}

fn validate_tiff(path: &Path) -> ValidationResult {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return ValidationResult::Invalid(format!("Cannot open: {}", e)),
    };
    let mut header = [0u8; 4];
    if file.read_exact(&mut header).is_err() {
        return ValidationResult::Invalid("File too small".to_string());
    }
    if (&header[0..2] == b"II" && header[2] == 0x2A && header[3] == 0x00)
        || (&header[0..2] == b"MM" && header[2] == 0x00 && header[3] == 0x2A)
    {
        ValidationResult::Valid
    } else {
        ValidationResult::Invalid("Invalid TIFF header".to_string())
    }
}

fn validate_text(path: &Path) -> ValidationResult {
    match fs::read(path) {
        Ok(data) => {
            if std::str::from_utf8(&data).is_ok() {
                ValidationResult::Valid
            } else {
                ValidationResult::ValidWithWarnings(vec![
                    "File contains non-UTF-8 bytes".to_string(),
                ])
            }
        }
        Err(e) => ValidationResult::Invalid(format!("Cannot read: {}", e)),
    }
}
