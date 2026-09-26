use super::error::ConversionError;
use super::request::ConversionOptions;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum JobStatus {
    Queued,
    Detecting,
    Planning,
    Preparing,
    Converting,
    Validating,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedFileInfo {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub detected_format: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub format_mismatch: bool,
    pub metadata: FileMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    pub page_count: Option<u32>,
    pub sheet_count: Option<u32>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub duration_ms: Option<u64>,
    pub has_text_layer: Option<bool>,
    pub is_password_protected: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionPlanStep {
    pub from: String,
    pub to: String,
    pub engine_id: String,
    pub engine_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionPlan {
    pub steps: Vec<ConversionPlanStep>,
    pub estimated_quality: f32,
    pub estimated_duration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionJob {
    pub id: String,
    pub input_file: DetectedFileInfo,
    pub output_format: String,
    pub output_path: String,
    pub options: ConversionOptions,
    pub plan: Option<ConversionPlan>,
    pub status: JobStatus,
    pub progress: f32,
    pub progress_message: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error: Option<ConversionError>,
    pub warnings: Vec<String>,
}

impl ConversionJob {
    pub fn new(
        id: String,
        input_file: DetectedFileInfo,
        output_format: String,
        output_path: String,
        options: ConversionOptions,
    ) -> Self {
        Self {
            id,
            input_file,
            output_format,
            output_path,
            options,
            plan: None,
            status: JobStatus::Queued,
            progress: 0.0,
            progress_message: String::new(),
            started_at: None,
            completed_at: None,
            error: None,
            warnings: Vec::new(),
        }
    }

    pub fn set_status(&mut self, status: JobStatus) {
        self.status = status;
        match &self.status {
            JobStatus::Converting => {
                if self.started_at.is_none() {
                    self.started_at = Some(Utc::now().to_rfc3339());
                }
            }
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => {
                self.completed_at = Some(Utc::now().to_rfc3339());
            }
            _ => {}
        }
    }

    pub fn fail(&mut self, error: ConversionError) {
        self.error = Some(error);
        self.set_status(JobStatus::Failed);
    }

    pub fn complete(&mut self, output_path: String) {
        self.output_path = output_path;
        self.progress = 100.0;
        self.set_status(JobStatus::Completed);
    }
}
