use super::extension::ExtensionResult;
use super::magic::MagicResult;

pub fn reconcile(
    magic: &MagicResult,
    ext: &ExtensionResult,
    raw_extension: &str,
) -> (String, String, bool) {
    if magic.format == "unknown" && ext.format == "unknown" {
        return ("unknown".to_string(), "application/octet-stream".to_string(), false);
    }

    if magic.format == "unknown" {
        return (ext.format.clone(), ext.mime.clone(), false);
    }

    if ext.format == "unknown" || raw_extension.is_empty() {
        return (magic.format.clone(), magic.mime.clone(), false);
    }

    if magic.format == ext.format {
        return (magic.format.clone(), magic.mime.clone(), false);
    }

    // ZIP-based formats: magic detected the subtype more specifically
    if ext.format == "zip" && ["docx", "xlsx", "pptx", "odt", "ods", "odp"].contains(&magic.format.as_str()) {
        return (magic.format.clone(), magic.mime.clone(), false);
    }

    // Text-based formats can't be detected by magic, trust extension
    let text_formats = [
        "txt", "csv", "tsv", "md", "py", "js", "ts", "java", "cpp", "c",
        "cs", "php", "rb", "go", "rs", "css", "sql", "sh", "yaml",
    ];
    if text_formats.contains(&ext.format.as_str()) && magic.format == "unknown" {
        return (ext.format.clone(), ext.mime.clone(), false);
    }

    // Genuine mismatch: magic says one thing, extension says another
    // Trust magic bytes but flag the mismatch
    (magic.format.clone(), magic.mime.clone(), true)
}
