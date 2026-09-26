use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConverterManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub engine_type: EngineType,
    pub license: String,
    pub input_formats: Vec<String>,
    pub output_formats: Vec<String>,
    pub platforms: Vec<Platform>,
    pub priority: u8,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineType {
    Bundled,
    External { executable: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else {
            Platform::Linux
        }
    }
}
