use std::path::Path;
use universal_file_converter::validators::{validate_output, ValidationResult};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn validate_pdf_valid() {
    match validate_output(&fixture("sample_text.pdf"), "pdf") {
        ValidationResult::Valid => {}
        other => panic!("Expected Valid, got {:?}", matches!(other, ValidationResult::Invalid(_))),
    }
}

#[test]
fn validate_png_valid() {
    match validate_output(&fixture("sample.png"), "png") {
        ValidationResult::Valid => {}
        other => panic!("Expected Valid, got {:?}", matches!(other, ValidationResult::Invalid(_))),
    }
}

#[test]
fn validate_nonexistent_file() {
    match validate_output(Path::new("/no/such/file.pdf"), "pdf") {
        ValidationResult::Invalid(msg) => {
            assert!(msg.contains("does not exist"));
        }
        _ => panic!("Expected Invalid"),
    }
}

#[test]
fn validate_empty_file() {
    let tmp = std::env::temp_dir().join("ufc_val_empty");
    std::fs::create_dir_all(&tmp).unwrap();
    let empty = tmp.join("empty.txt");
    std::fs::write(&empty, "").unwrap();

    match validate_output(&empty, "txt") {
        ValidationResult::Invalid(msg) => {
            assert!(msg.contains("empty"));
        }
        _ => panic!("Expected Invalid for empty file"),
    }

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn validate_txt_utf8() {
    match validate_output(&fixture("sample.txt"), "txt") {
        ValidationResult::Valid => {}
        _ => panic!("Expected Valid for UTF-8 text"),
    }
}

#[test]
fn validate_csv_valid() {
    match validate_output(&fixture("sample.csv"), "csv") {
        ValidationResult::Valid => {}
        _ => panic!("Expected Valid for CSV"),
    }
}

#[test]
fn validate_xlsx_valid() {
    match validate_output(&fixture("sample.xlsx"), "xlsx") {
        ValidationResult::Valid => {}
        other => panic!("Expected Valid for XLSX, got {:?}", matches!(other, ValidationResult::Invalid(_))),
    }
}

#[test]
fn validate_wrong_magic_bytes() {
    let tmp = std::env::temp_dir().join("ufc_val_wrong");
    std::fs::create_dir_all(&tmp).unwrap();
    let fake_png = tmp.join("fake.png");
    std::fs::write(&fake_png, b"this is not a png").unwrap();

    match validate_output(&fake_png, "png") {
        ValidationResult::Invalid(msg) => {
            assert!(msg.contains("Invalid"));
        }
        _ => panic!("Expected Invalid for wrong magic bytes"),
    }

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn validate_unknown_format_passes() {
    let tmp = std::env::temp_dir().join("ufc_val_unknown");
    std::fs::create_dir_all(&tmp).unwrap();
    let file = tmp.join("data.xyz");
    std::fs::write(&file, b"some data").unwrap();

    match validate_output(&file, "xyz") {
        ValidationResult::Valid => {}
        _ => panic!("Expected Valid for unknown format"),
    }

    let _ = std::fs::remove_dir_all(&tmp);
}
