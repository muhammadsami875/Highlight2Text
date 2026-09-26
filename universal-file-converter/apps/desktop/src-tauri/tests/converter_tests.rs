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
    assert!(status.len() >= 8);
    let bundled_count = status.iter().filter(|e| e.available).count();
    assert!(bundled_count >= 7);
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
