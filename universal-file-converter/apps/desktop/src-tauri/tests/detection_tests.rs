use std::path::Path;
use universal_file_converter::detectors;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

// ─── Magic byte detection ───

#[test]
fn detect_png_by_magic() {
    let result = detectors::detect_format(&fixture("sample.png")).unwrap();
    assert_eq!(result.detected_format, "png");
    assert_eq!(result.mime_type, "image/png");
    assert!(!result.format_mismatch);
}

#[test]
fn detect_jpeg_by_magic() {
    let result = detectors::detect_format(&fixture("sample.jpg")).unwrap();
    assert_eq!(result.detected_format, "jpg");
    assert_eq!(result.mime_type, "image/jpeg");
}

#[test]
fn detect_bmp_by_magic() {
    let result = detectors::detect_format(&fixture("sample.bmp")).unwrap();
    assert_eq!(result.detected_format, "bmp");
    assert_eq!(result.mime_type, "image/bmp");
}

#[test]
fn detect_gif_by_magic() {
    let result = detectors::detect_format(&fixture("sample.gif")).unwrap();
    assert_eq!(result.detected_format, "gif");
    assert_eq!(result.mime_type, "image/gif");
}

#[test]
fn detect_webp_by_magic() {
    let result = detectors::detect_format(&fixture("sample.webp")).unwrap();
    assert_eq!(result.detected_format, "webp");
    assert_eq!(result.mime_type, "image/webp");
}

#[test]
fn detect_tiff_little_endian() {
    let result = detectors::detect_format(&fixture("sample.tiff")).unwrap();
    assert_eq!(result.detected_format, "tiff");
    assert_eq!(result.mime_type, "image/tiff");
}

#[test]
fn detect_tiff_big_endian() {
    let result = detectors::detect_format(&fixture("sample_be.tiff")).unwrap();
    assert_eq!(result.detected_format, "tiff");
}

#[test]
fn detect_pdf_by_magic() {
    let result = detectors::detect_format(&fixture("sample.pdf")).unwrap();
    assert_eq!(result.detected_format, "pdf");
    assert_eq!(result.mime_type, "application/pdf");
}

#[test]
fn detect_rtf_by_magic() {
    let result = detectors::detect_format(&fixture("sample.rtf")).unwrap();
    assert_eq!(result.detected_format, "rtf");
    assert_eq!(result.mime_type, "application/rtf");
}

// ─── Text-based detection ───

#[test]
fn detect_html_by_content() {
    let result = detectors::detect_format(&fixture("sample.html")).unwrap();
    assert_eq!(result.detected_format, "html");
    assert_eq!(result.mime_type, "text/html");
}

#[test]
fn detect_xml_by_content() {
    let result = detectors::detect_format(&fixture("sample.xml")).unwrap();
    assert_eq!(result.detected_format, "xml");
    assert_eq!(result.mime_type, "application/xml");
}

#[test]
fn detect_json_by_content() {
    let result = detectors::detect_format(&fixture("sample.json")).unwrap();
    assert_eq!(result.detected_format, "json");
    assert_eq!(result.mime_type, "application/json");
}

#[test]
fn detect_csv_by_heuristic() {
    let result = detectors::detect_format(&fixture("sample.csv")).unwrap();
    assert_eq!(result.detected_format, "csv");
}

#[test]
fn detect_markdown_by_heuristic() {
    let result = detectors::detect_format(&fixture("sample.md")).unwrap();
    assert_eq!(result.detected_format, "md");
}

// ─── Extension-based detection (text formats with no magic) ───

#[test]
fn detect_txt_by_extension() {
    let result = detectors::detect_format(&fixture("sample.txt")).unwrap();
    assert_eq!(result.detected_format, "txt");
}

#[test]
fn detect_python_by_extension() {
    let result = detectors::detect_format(&fixture("sample.py")).unwrap();
    assert_eq!(result.detected_format, "py");
}

#[test]
fn detect_javascript_by_extension() {
    let result = detectors::detect_format(&fixture("sample.js")).unwrap();
    assert_eq!(result.detected_format, "js");
}

#[test]
fn detect_css_by_extension() {
    let result = detectors::detect_format(&fixture("sample.css")).unwrap();
    assert_eq!(result.detected_format, "css");
}

#[test]
fn detect_sql_by_extension() {
    let result = detectors::detect_format(&fixture("sample.sql")).unwrap();
    assert_eq!(result.detected_format, "sql");
}

// ─── ZIP-based format disambiguation ───

#[test]
fn detect_docx_from_zip() {
    let result = detectors::detect_format(&fixture("sample.docx")).unwrap();
    assert_eq!(result.detected_format, "docx");
    assert!(result.mime_type.contains("wordprocessing"));
}

#[test]
fn detect_xlsx_from_zip() {
    let result = detectors::detect_format(&fixture("sample.xlsx")).unwrap();
    assert_eq!(result.detected_format, "xlsx");
    assert!(result.mime_type.contains("spreadsheet"));
}

#[test]
fn detect_pptx_from_zip() {
    let result = detectors::detect_format(&fixture("sample.pptx")).unwrap();
    assert_eq!(result.detected_format, "pptx");
    assert!(result.mime_type.contains("presentation"));
}

#[test]
fn detect_odt_from_zip() {
    let result = detectors::detect_format(&fixture("sample.odt")).unwrap();
    assert_eq!(result.detected_format, "odt");
}

#[test]
fn detect_ods_from_zip() {
    let result = detectors::detect_format(&fixture("sample.ods")).unwrap();
    assert_eq!(result.detected_format, "ods");
}

#[test]
fn detect_plain_zip() {
    let result = detectors::detect_format(&fixture("sample.zip")).unwrap();
    assert_eq!(result.detected_format, "zip");
}

// ─── Format mismatch detection ───

#[test]
fn detect_mismatch_png_with_txt_extension() {
    let result = detectors::detect_format(&fixture("fake.txt")).unwrap();
    assert_eq!(result.detected_format, "png");
    assert!(result.format_mismatch);
}

// ─── Edge cases ───

#[test]
fn detect_empty_file() {
    let result = detectors::detect_format(&fixture("empty.bin"));
    assert!(result.is_ok());
    let r = result.unwrap();
    assert_eq!(r.detected_format, "unknown");
    assert_eq!(r.size_bytes, 0);
}

#[test]
fn detect_truncated_png() {
    let result = detectors::detect_format(&fixture("truncated.png")).unwrap();
    // Truncated — not enough bytes for full PNG magic, falls back
    // Should not crash, format detected or unknown
    assert!(!result.detected_format.is_empty());
}

#[test]
fn detect_nonexistent_file() {
    let result = detectors::detect_format(Path::new("/nonexistent/file.pdf"));
    assert!(result.is_err());
}

// ─── Metadata extraction ───

#[test]
fn png_metadata_has_dimensions() {
    let result = detectors::detect_format(&fixture("sample.png")).unwrap();
    assert_eq!(result.metadata.image_width, Some(1));
    assert_eq!(result.metadata.image_height, Some(1));
}

#[test]
fn pdf_metadata_has_page_count() {
    let result = detectors::detect_format(&fixture("sample.pdf")).unwrap();
    assert!(result.metadata.page_count.is_some());
}

// ─── Extension module tests ───

#[test]
fn extension_all_known_formats() {
    let known = vec![
        ("pdf", "pdf"), ("doc", "doc"), ("docx", "docx"),
        ("xls", "xls"), ("xlsx", "xlsx"), ("ppt", "ppt"),
        ("pptx", "pptx"), ("odt", "odt"), ("ods", "ods"),
        ("odp", "odp"), ("rtf", "rtf"), ("txt", "txt"),
        ("csv", "csv"), ("tsv", "tsv"), ("md", "md"),
        ("html", "html"), ("htm", "html"), ("xml", "xml"),
        ("json", "json"), ("png", "png"), ("jpg", "jpg"),
        ("jpeg", "jpg"), ("webp", "webp"), ("gif", "gif"),
        ("bmp", "bmp"), ("tiff", "tiff"), ("tif", "tiff"),
        ("svg", "svg"), ("py", "py"), ("js", "js"),
        ("ts", "ts"), ("java", "java"), ("cpp", "cpp"),
        ("c", "c"), ("cs", "cs"), ("php", "php"),
        ("rb", "rb"), ("go", "go"), ("rs", "rs"),
        ("css", "css"), ("sql", "sql"), ("sh", "sh"),
        ("yaml", "yaml"), ("yml", "yaml"),
    ];

    for (ext, expected) in known {
        let result = detectors::extension::detect_by_extension(ext);
        assert_eq!(
            result.format, expected,
            "Extension '{}' should map to format '{}'",
            ext, expected
        );
        assert_ne!(result.mime, "application/octet-stream",
            "Extension '{}' should have a specific MIME type", ext);
    }
}

#[test]
fn extension_unknown_returns_unknown() {
    let result = detectors::extension::detect_by_extension("xyz123");
    assert_eq!(result.format, "unknown");
    assert_eq!(result.mime, "application/octet-stream");
}

// ─── Reconciliation tests ───

#[test]
fn reconcile_matching_formats() {
    let magic = detectors::magic::MagicResult {
        format: "pdf".into(),
        mime: "application/pdf".into(),
    };
    let ext = detectors::extension::ExtensionResult {
        format: "pdf".into(),
        mime: "application/pdf".into(),
    };
    let (fmt, _mime, mismatch) = detectors::reconcile::reconcile(&magic, &ext, "pdf");
    assert_eq!(fmt, "pdf");
    assert!(!mismatch);
}

#[test]
fn reconcile_magic_wins_on_mismatch() {
    let magic = detectors::magic::MagicResult {
        format: "png".into(),
        mime: "image/png".into(),
    };
    let ext = detectors::extension::ExtensionResult {
        format: "txt".into(),
        mime: "text/plain".into(),
    };
    let (fmt, _mime, mismatch) = detectors::reconcile::reconcile(&magic, &ext, "txt");
    assert_eq!(fmt, "png");
    assert!(mismatch);
}

#[test]
fn reconcile_zip_subtype_no_mismatch() {
    let magic = detectors::magic::MagicResult {
        format: "docx".into(),
        mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".into(),
    };
    let ext = detectors::extension::ExtensionResult {
        format: "zip".into(),
        mime: "application/zip".into(),
    };
    let (fmt, _mime, mismatch) = detectors::reconcile::reconcile(&magic, &ext, "zip");
    assert_eq!(fmt, "docx");
    assert!(!mismatch);
}

#[test]
fn reconcile_unknown_magic_uses_extension() {
    let magic = detectors::magic::MagicResult {
        format: "unknown".into(),
        mime: "application/octet-stream".into(),
    };
    let ext = detectors::extension::ExtensionResult {
        format: "py".into(),
        mime: "text/x-python".into(),
    };
    let (fmt, _mime, mismatch) = detectors::reconcile::reconcile(&magic, &ext, "py");
    assert_eq!(fmt, "py");
    assert!(!mismatch);
}

#[test]
fn reconcile_both_unknown() {
    let magic = detectors::magic::MagicResult {
        format: "unknown".into(),
        mime: "application/octet-stream".into(),
    };
    let ext = detectors::extension::ExtensionResult {
        format: "unknown".into(),
        mime: "application/octet-stream".into(),
    };
    let (fmt, _mime, mismatch) = detectors::reconcile::reconcile(&magic, &ext, "");
    assert_eq!(fmt, "unknown");
    assert!(!mismatch);
}

#[test]
fn reconcile_no_extension_uses_magic() {
    let magic = detectors::magic::MagicResult {
        format: "pdf".into(),
        mime: "application/pdf".into(),
    };
    let ext = detectors::extension::ExtensionResult {
        format: "unknown".into(),
        mime: "application/octet-stream".into(),
    };
    let (fmt, _mime, mismatch) = detectors::reconcile::reconcile(&magic, &ext, "");
    assert_eq!(fmt, "pdf");
    assert!(!mismatch);
}

// ─── DetectedFileInfo fields ───

#[test]
fn detected_file_info_fields_populated() {
    let result = detectors::detect_format(&fixture("sample.png")).unwrap();
    assert!(result.path.contains("sample.png"));
    assert_eq!(result.name, "sample.png");
    assert_eq!(result.extension, "png");
    assert!(result.size_bytes > 0);
}
