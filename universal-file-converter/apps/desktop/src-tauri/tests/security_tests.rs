use universal_file_converter::security;
use universal_file_converter::filesystem;

#[test]
fn sanitize_filename_removes_path_separators() {
    let sanitized = security::sanitize_filename("../../etc/passwd");
    assert!(!sanitized.contains('/'));
    assert!(!sanitized.contains('\\'));
}

#[test]
fn sanitize_filename_removes_special_chars() {
    let sanitized = security::sanitize_filename("file:name*with?bad<chars>.txt");
    assert!(!sanitized.contains(':'));
    assert!(!sanitized.contains('*'));
    assert!(!sanitized.contains('?'));
    assert!(!sanitized.contains('<'));
    assert!(!sanitized.contains('>'));
}

#[test]
fn sanitize_filename_strips_leading_dots() {
    let sanitized = security::sanitize_filename(".hidden");
    assert!(!sanitized.starts_with('.'));
    assert_eq!(sanitized, "hidden");
}

#[test]
fn sanitize_filename_removes_null_bytes() {
    let sanitized = security::sanitize_filename("file\0name.txt");
    assert!(!sanitized.contains('\0'));
}

#[test]
fn sanitize_filename_handles_control_chars() {
    let sanitized = security::sanitize_filename("file\x01\x02name");
    assert!(!sanitized.chars().any(|c| c.is_control()));
}

#[test]
fn sanitize_filename_preserves_unicode() {
    let sanitized = security::sanitize_filename("تقرير_٢٠٢٤.pdf");
    assert_eq!(sanitized, "تقرير_٢٠٢٤.pdf");
}

#[test]
fn sanitize_path_rejects_traversal() {
    let tmp = std::env::temp_dir().join("ufc_sec_test");
    std::fs::create_dir_all(&tmp).unwrap();

    let result = security::sanitize_path("../../etc/passwd", &tmp);
    assert!(result.is_err());
}

#[test]
fn sanitize_path_accepts_valid_child() {
    let tmp = std::env::temp_dir().join("ufc_sec_test_valid");
    std::fs::create_dir_all(&tmp).unwrap();

    let child = tmp.join("test.txt");
    std::fs::write(&child, "test").unwrap();

    let result = security::sanitize_path(child.to_str().unwrap(), &tmp);
    assert!(result.is_ok());

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn validate_file_size_rejects_too_large() {
    let tmp = std::env::temp_dir().join("ufc_size_test");
    std::fs::create_dir_all(&tmp).unwrap();
    let file = tmp.join("small.txt");
    std::fs::write(&file, "hello").unwrap();

    let result = security::validate_file_size(&file, 3);
    assert!(result.is_err());

    let result = security::validate_file_size(&file, 100);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn generate_output_avoids_collision() {
    let tmp = std::env::temp_dir().join("ufc_filename_test");
    std::fs::create_dir_all(&tmp).unwrap();

    let first = filesystem::generate_output_filename("test.txt", "pdf", "-converted", &tmp);
    std::fs::write(&first, "fake").unwrap();

    let second = filesystem::generate_output_filename("test.txt", "pdf", "-converted", &tmp);
    assert_ne!(first, second);
    assert!(second.to_string_lossy().contains("-1"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn generate_output_sanitizes_malicious_name() {
    let tmp = std::env::temp_dir().join("ufc_malname_test");
    std::fs::create_dir_all(&tmp).unwrap();

    let path = filesystem::generate_output_filename(
        "../../etc/evil.txt",
        "pdf",
        "-converted",
        &tmp,
    );
    let filename = path.file_name().unwrap().to_str().unwrap();
    assert!(!filename.contains('/'));
    assert!(!filename.contains(".."));

    let _ = std::fs::remove_dir_all(&tmp);
}
