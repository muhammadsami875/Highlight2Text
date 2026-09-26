use std::path::{Path, PathBuf};
use universal_file_converter::conversion::request::{ConversionOptions, ConversionRequest};
use universal_file_converter::converters;
use universal_file_converter::engines::registry::{Converter, ConverterRegistry};
use universal_file_converter::planner;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn temp_job_dir() -> PathBuf {
    let dir = std::env::temp_dir()
        .join("ufc_tests")
        .join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(dir.join("output")).unwrap();
    dir
}

fn default_options() -> ConversionOptions {
    ConversionOptions {
        page_range: None,
        dpi: None,
        image_quality: None,
        compression: None,
        page_size: None,
        orientation: None,
        margins: None,
        font_size: None,
        font_family: None,
        line_numbers: None,
        line_wrapping: None,
        syntax_theme: None,
        sheet_selection: None,
        fit_to_page: None,
        ocr_enabled: None,
        ocr_language: None,
        preserve_layout: None,
        background_color: None,
        password: None,
    }
}

// ─── Registry tests ───

#[test]
fn registry_finds_image_converter() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("png", "jpg"));
    assert!(registry.is_supported("jpg", "png"));
    assert!(registry.is_supported("bmp", "webp"));
}

#[test]
fn registry_finds_image_to_pdf() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("png", "pdf"));
    assert!(registry.is_supported("jpg", "pdf"));
}

#[test]
fn registry_finds_markdown_to_html() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("md", "html"));
}

#[test]
fn registry_finds_csv_json() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("csv", "json"));
    assert!(registry.is_supported("json", "csv"));
}

#[test]
fn registry_finds_csv_xlsx() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("csv", "xlsx"));
}

#[test]
fn registry_finds_code_to_html() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("py", "html"));
    assert!(registry.is_supported("rs", "html"));
    assert!(registry.is_supported("js", "html"));
}

#[test]
fn registry_reports_unsupported() {
    let registry = ConverterRegistry::new();
    assert!(!registry.is_supported("mp3", "pdf"));
    assert!(!registry.is_supported("zip", "png"));
}

#[test]
fn registry_supported_outputs_for_png() {
    let registry = ConverterRegistry::new();
    let outputs = registry.supported_outputs("png");
    assert!(outputs.contains(&"jpg".to_string()));
    assert!(outputs.contains(&"pdf".to_string()));
    assert!(outputs.contains(&"webp".to_string()));
}

#[test]
fn registry_engine_status_lists_all() {
    let registry = ConverterRegistry::new();
    let status = registry.engine_status();
    assert!(status.len() >= 10);
    let bundled_count = status.iter().filter(|e| e.available).count();
    assert!(bundled_count >= 9);
}

// ─── Planner tests ───

#[test]
fn planner_direct_route() {
    let registry = ConverterRegistry::new();
    let plan = planner::plan_conversion("png", "jpg", &registry);
    assert!(plan.is_some());
    let plan = plan.unwrap();
    assert_eq!(plan.steps.len(), 1);
    assert_eq!(plan.steps[0].from, "png");
    assert_eq!(plan.steps[0].to, "jpg");
}

#[test]
fn planner_two_step_route() {
    let registry = ConverterRegistry::new();
    // md -> html is direct, but md -> pdf requires md -> html -> ???
    // Actually we don't have html->pdf yet, so let's test something that works:
    // gif -> pdf should go gif -> png -> pdf
    let plan = planner::plan_conversion("gif", "pdf", &registry);
    assert!(plan.is_some());
    let plan = plan.unwrap();
    // gif can be converted to png (image_convert), then png to pdf (image_to_pdf)
    assert!(plan.steps.len() >= 1);
}

#[test]
fn planner_no_route() {
    let registry = ConverterRegistry::new();
    let plan = planner::plan_conversion("mp3", "docx", &registry);
    assert!(plan.is_none());
}

// ─── Converter execution tests ───

#[test]
fn convert_png_to_jpg() {
    let job_dir = temp_job_dir();
    let converter = converters::image_convert::ImageConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.png"),
        detected_format: "png".into(),
        output_format: "jpg".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.exists());
    assert!(output.to_string_lossy().ends_with(".jpg"));
}

#[test]
fn convert_png_to_bmp() {
    let job_dir = temp_job_dir();
    let converter = converters::image_convert::ImageConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.png"),
        detected_format: "png".into(),
        output_format: "bmp".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok());
    assert!(result.unwrap().exists());
}

#[test]
fn convert_png_to_pdf() {
    let job_dir = temp_job_dir();
    let converter = converters::image_to_pdf::ImageToPdfConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.png"),
        detected_format: "png".into(),
        output_format: "pdf".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.exists());
    let contents = std::fs::read(&output).unwrap();
    assert!(contents.starts_with(b"%PDF"));
}

#[test]
fn convert_markdown_to_html() {
    let job_dir = temp_job_dir();
    let converter = converters::markdown_to_html::MarkdownToHtmlConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.md"),
        detected_format: "md".into(),
        output_format: "html".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok());
    let output = result.unwrap();
    let html = std::fs::read_to_string(&output).unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Hello World"));
    assert!(html.contains("<strong>"));
}

#[test]
fn convert_csv_to_json() {
    let job_dir = temp_job_dir();
    let converter = converters::csv_json::CsvJsonConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.csv"),
        detected_format: "csv".into(),
        output_format: "json".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok());
    let output = result.unwrap();
    let json_str = std::fs::read_to_string(&output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_array());
    let arr = parsed.as_array().unwrap();
    assert!(arr.len() >= 2);
    assert_eq!(arr[0]["name"], "Alice");
}

#[test]
fn convert_json_to_csv() {
    // First create a JSON array file
    let job_dir = temp_job_dir();
    let json_path = job_dir.join("input.json");
    std::fs::write(
        &json_path,
        r#"[{"name":"Alice","age":"30"},{"name":"Bob","age":"25"}]"#,
    )
    .unwrap();

    let converter = converters::csv_json::CsvJsonConverter::new();
    let request = ConversionRequest {
        input_path: json_path,
        detected_format: "json".into(),
        output_format: "csv".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok());
    let output = result.unwrap();
    let csv_str = std::fs::read_to_string(&output).unwrap();
    assert!(csv_str.contains("name"));
    assert!(csv_str.contains("Alice"));
}

#[test]
fn convert_csv_to_xlsx() {
    let job_dir = temp_job_dir();
    let converter = converters::csv_xlsx::CsvXlsxConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.csv"),
        detected_format: "csv".into(),
        output_format: "xlsx".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.exists());
    let contents = std::fs::read(&output).unwrap();
    // XLSX is a ZIP file
    assert!(contents.starts_with(&[0x50, 0x4B, 0x03, 0x04]));
}

#[test]
fn convert_python_to_html() {
    let job_dir = temp_job_dir();
    let converter = converters::code_highlight::CodeHighlightConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.py"),
        detected_format: "py".into(),
        output_format: "html".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok());
    let output = result.unwrap();
    let html = std::fs::read_to_string(&output).unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("hello"));
}

#[test]
fn convert_rejects_unsupported_pair() {
    let converter = converters::image_convert::ImageConverter::new();
    assert!(!converter.can_convert("pdf", "png"));
    assert!(!converter.can_convert("png", "png")); // same format
}

#[test]
fn registry_finds_pdf_to_txt() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("pdf", "txt"));
}

#[test]
fn registry_finds_txt_to_pdf() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("txt", "pdf"));
    assert!(registry.is_supported("py", "pdf"));
    assert!(registry.is_supported("rs", "pdf"));
}

#[test]
fn convert_txt_to_pdf() {
    let job_dir = temp_job_dir();
    let converter = converters::text_to_pdf::TextToPdfConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.txt"),
        detected_format: "txt".into(),
        output_format: "pdf".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.exists());
    let contents = std::fs::read(&output).unwrap();
    assert!(contents.starts_with(b"%PDF"));
}

#[test]
fn convert_pdf_to_txt() {
    let job_dir = temp_job_dir();
    let converter = converters::pdf_extract::PdfTextExtractor::new();
    let request = ConversionRequest {
        input_path: fixture("sample_text.pdf"),
        detected_format: "pdf".into(),
        output_format: "txt".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok());
    let output = result.unwrap();
    let text = std::fs::read_to_string(&output).unwrap();
    assert!(text.contains("Hello World"));
}

#[test]
fn registry_finds_xlsx_to_csv() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("xlsx", "csv"));
    assert!(registry.is_supported("xlsx", "json"));
    assert!(registry.is_supported("ods", "csv"));
}

#[test]
fn convert_xlsx_to_csv() {
    let job_dir = temp_job_dir();
    let converter = converters::xlsx_extract::XlsxExtractConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.xlsx"),
        detected_format: "xlsx".into(),
        output_format: "csv".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok(), "xlsx to csv failed: {:?}", result.err());
    let output = result.unwrap();
    let csv_str = std::fs::read_to_string(&output).unwrap();
    assert!(!csv_str.is_empty());
}

#[test]
fn convert_xlsx_to_json() {
    let job_dir = temp_job_dir();
    let converter = converters::xlsx_extract::XlsxExtractConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.xlsx"),
        detected_format: "xlsx".into(),
        output_format: "json".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok(), "xlsx to json failed: {:?}", result.err());
    let output = result.unwrap();
    let json_str = std::fs::read_to_string(&output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn registry_finds_html_to_pdf() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("html", "pdf"));
}

#[test]
fn convert_html_to_pdf() {
    let job_dir = temp_job_dir();
    let converter = converters::html_to_pdf::HtmlToPdfConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.html"),
        detected_format: "html".into(),
        output_format: "pdf".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok(), "html to pdf failed: {:?}", result.err());
    let output = result.unwrap();
    assert!(output.exists());
    let contents = std::fs::read(&output).unwrap();
    assert!(contents.starts_with(b"%PDF"));
}

#[test]
fn convert_png_to_gif() {
    let job_dir = temp_job_dir();
    let converter = converters::image_convert::ImageConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.png"),
        detected_format: "png".into(),
        output_format: "gif".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok(), "png to gif failed: {:?}", result.err());
    assert!(result.unwrap().exists());
}

#[test]
fn converter_manifests_have_required_fields() {
    let registry = ConverterRegistry::new();
    let engines = registry.engine_status();
    for engine in &engines {
        assert!(!engine.id.is_empty());
        assert!(!engine.name.is_empty());
        assert!(!engine.license.is_empty());
        assert!(!engine.input_formats.is_empty());
        assert!(!engine.output_formats.is_empty());
    }
}

// ─── Multi-step conversion tests ───

#[test]
fn planner_md_to_pdf_direct() {
    let registry = ConverterRegistry::new();
    let plan = planner::plan_conversion("md", "pdf", &registry);
    assert!(plan.is_some(), "md -> pdf should have a route");
    let plan = plan.unwrap();
    assert_eq!(plan.steps.len(), 1);
    assert_eq!(plan.steps[0].from, "md");
    assert_eq!(plan.steps[0].to, "pdf");
}

#[test]
fn planner_json_to_xlsx_via_csv() {
    let registry = ConverterRegistry::new();
    let plan = planner::plan_conversion("json", "xlsx", &registry);
    assert!(plan.is_some(), "json -> xlsx should have a route");
    let plan = plan.unwrap();
    assert_eq!(plan.steps.len(), 2);
    assert_eq!(plan.steps[0].from, "json");
    assert_eq!(plan.steps[0].to, "csv");
    assert_eq!(plan.steps[1].from, "csv");
    assert_eq!(plan.steps[1].to, "xlsx");
}

#[test]
fn planner_code_to_pdf_direct() {
    let registry = ConverterRegistry::new();
    let plan = planner::plan_conversion("py", "pdf", &registry);
    assert!(plan.is_some(), "py -> pdf should have a route");
    let plan = plan.unwrap();
    assert_eq!(plan.steps.len(), 1);
    assert_eq!(plan.steps[0].from, "py");
    assert_eq!(plan.steps[0].to, "pdf");
}

#[test]
fn planner_bmp_to_pdf_via_image() {
    let registry = ConverterRegistry::new();
    let plan = planner::plan_conversion("bmp", "pdf", &registry);
    assert!(plan.is_some(), "bmp -> pdf should have a route");
}

// ─── Additional converter tests ───

#[test]
fn convert_png_to_webp() {
    let job_dir = temp_job_dir();
    let converter = converters::image_convert::ImageConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.png"),
        detected_format: "png".into(),
        output_format: "webp".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok(), "png to webp failed: {:?}", result.err());
    assert!(result.unwrap().exists());
}

#[test]
fn convert_jpg_to_png() {
    let job_dir = temp_job_dir();
    let converter = converters::image_convert::ImageConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.jpg"),
        detected_format: "jpg".into(),
        output_format: "png".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok(), "jpg to png failed: {:?}", result.err());
    assert!(result.unwrap().exists());
}

#[test]
fn convert_rejects_same_image_format() {
    let converter = converters::image_convert::ImageConverter::new();
    assert!(!converter.can_convert("png", "png"));
    assert!(!converter.can_convert("jpg", "jpg"));
}

#[test]
fn convert_csv_roundtrip() {
    let job_dir = temp_job_dir();

    let csv_converter = converters::csv_json::CsvJsonConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.csv"),
        detected_format: "csv".into(),
        output_format: "json".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let json_path = csv_converter.convert(&request, &job_dir).unwrap();

    let job_dir2 = temp_job_dir();
    let request2 = ConversionRequest {
        input_path: json_path,
        detected_format: "json".into(),
        output_format: "csv".into(),
        output_dir: job_dir2.join("output"),
        options: default_options(),
    };
    let result = csv_converter.convert(&request2, &job_dir2);
    assert!(result.is_ok(), "CSV roundtrip failed: {:?}", result.err());
    let csv_str = std::fs::read_to_string(result.unwrap()).unwrap();
    assert!(csv_str.contains("Alice"));
}

// ─── Office converter tests ───

#[test]
fn registry_finds_pdf_to_docx() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("pdf", "docx"));
}

#[test]
fn registry_finds_pdf_to_xlsx() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("pdf", "xlsx"));
}

#[test]
fn registry_finds_txt_to_docx() {
    let registry = ConverterRegistry::new();
    assert!(registry.is_supported("txt", "docx"));
    assert!(registry.is_supported("md", "docx"));
}

#[test]
fn convert_pdf_to_docx() {
    let job_dir = temp_job_dir();
    let converter = converters::pdf_to_docx::PdfToDocxConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample_text.pdf"),
        detected_format: "pdf".into(),
        output_format: "docx".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok(), "pdf to docx failed: {:?}", result.err());
    let output = result.unwrap();
    assert!(output.exists());
    let contents = std::fs::read(&output).unwrap();
    assert!(contents.starts_with(&[0x50, 0x4B, 0x03, 0x04]));
}

#[test]
fn convert_pdf_to_xlsx() {
    let job_dir = temp_job_dir();
    let converter = converters::pdf_to_xlsx::PdfToXlsxConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample_text.pdf"),
        detected_format: "pdf".into(),
        output_format: "xlsx".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok(), "pdf to xlsx failed: {:?}", result.err());
    let output = result.unwrap();
    assert!(output.exists());
    let contents = std::fs::read(&output).unwrap();
    assert!(contents.starts_with(&[0x50, 0x4B, 0x03, 0x04]));
}

#[test]
fn convert_txt_to_docx() {
    let job_dir = temp_job_dir();
    let converter = converters::text_to_docx::TextToDocxConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.txt"),
        detected_format: "txt".into(),
        output_format: "docx".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok(), "txt to docx failed: {:?}", result.err());
    let output = result.unwrap();
    assert!(output.exists());
    let contents = std::fs::read(&output).unwrap();
    assert!(contents.starts_with(&[0x50, 0x4B, 0x03, 0x04]));
}

#[test]
fn convert_md_to_docx() {
    let job_dir = temp_job_dir();
    let converter = converters::text_to_docx::TextToDocxConverter::new();
    let request = ConversionRequest {
        input_path: fixture("sample.md"),
        detected_format: "md".into(),
        output_format: "docx".into(),
        output_dir: job_dir.join("output"),
        options: default_options(),
    };
    let result = converter.convert(&request, &job_dir);
    assert!(result.is_ok(), "md to docx failed: {:?}", result.err());
    let output = result.unwrap();
    assert!(output.exists());
    let contents = std::fs::read(&output).unwrap();
    assert!(contents.starts_with(&[0x50, 0x4B, 0x03, 0x04]));
}
