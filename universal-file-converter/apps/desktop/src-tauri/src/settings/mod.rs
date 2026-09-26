use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub conversion: ConversionSettings,
    pub pdf: PdfSettings,
    pub images: ImageSettings,
    pub ocr: OcrSettings,
    pub advanced: AdvancedSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub theme: String,
    pub language: String,
    pub start_minimized: bool,
    pub show_in_system_tray: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionSettings {
    pub output_dir_mode: String,
    pub custom_output_dir: String,
    pub overwrite_mode: String,
    pub file_suffix: String,
    pub temp_dir: String,
    pub max_parallel_jobs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfSettings {
    pub default_dpi: u32,
    pub default_compression: String,
    pub default_page_size: String,
    pub default_orientation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSettings {
    pub default_quality: u32,
    pub default_format: String,
    pub preserve_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrSettings {
    pub language: String,
    pub engine: String,
    pub auto_detect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedSettings {
    pub enable_experimental: bool,
    pub enable_diagnostic_logs: bool,
    pub max_file_size_mb: u64,
    pub process_timeout_sec: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            general: GeneralSettings {
                theme: "system".to_string(),
                language: "en".to_string(),
                start_minimized: false,
                show_in_system_tray: false,
            },
            conversion: ConversionSettings {
                output_dir_mode: "same".to_string(),
                custom_output_dir: String::new(),
                overwrite_mode: "rename".to_string(),
                file_suffix: "-converted".to_string(),
                temp_dir: String::new(),
                max_parallel_jobs: 2,
            },
            pdf: PdfSettings {
                default_dpi: 150,
                default_compression: "auto".to_string(),
                default_page_size: "A4".to_string(),
                default_orientation: "portrait".to_string(),
            },
            images: ImageSettings {
                default_quality: 85,
                default_format: "png".to_string(),
                preserve_metadata: true,
            },
            ocr: OcrSettings {
                language: "eng".to_string(),
                engine: "tesseract".to_string(),
                auto_detect: true,
            },
            advanced: AdvancedSettings {
                enable_experimental: false,
                enable_diagnostic_logs: false,
                max_file_size_mb: 2048,
                process_timeout_sec: 300,
            },
        }
    }
}

fn settings_path() -> PathBuf {
    dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("UniversalFileConverter")
        .join("settings.json")
}

pub fn load_settings() -> AppSettings {
    let path = settings_path();
    if !path.exists() {
        return AppSettings::default();
    }
    let Ok(data) = fs::read_to_string(&path) else {
        return AppSettings::default();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_settings_to_disk(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Cannot create config dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Cannot serialize settings: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Cannot write settings: {}", e))?;
    Ok(())
}
