use super::error::ConversionError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionResult {
    pub job_id: String,
    pub success: bool,
    pub output_path: Option<String>,
    pub output_size: Option<u64>,
    pub duration: u64,
    pub warnings: Vec<String>,
    pub error: Option<ConversionError>,
}
