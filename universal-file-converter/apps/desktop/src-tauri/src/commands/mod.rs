use crate::conversion::job::{ConversionJob, ConversionPlan, DetectedFileInfo, JobStatus};
use crate::conversion::request::ConversionOptions;
use crate::conversion::result::ConversionResult;
use crate::detectors;
use crate::engines::registry::EngineInfo;
use crate::jobs::worker;
use crate::planner;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub job_id: String,
    pub progress: f32,
    pub message: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedOutput {
    pub format: String,
    pub level: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProgress {
    pub total_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
    pub cancelled_jobs: usize,
    pub current_job_id: Option<String>,
    pub current_job_name: Option<String>,
    pub overall_progress: f32,
    pub elapsed_ms: u64,
    pub estimated_remaining_ms: Option<u64>,
}

#[tauri::command]
pub fn detect_file(path: String) -> Result<DetectedFileInfo, String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err("File does not exist".to_string());
    }
    detectors::detect_format(p)
}

#[tauri::command]
pub fn get_supported_outputs(
    input_format: String,
    state: State<'_, AppState>,
) -> Vec<SupportedOutput> {
    let outputs = state.registry.supported_outputs(&input_format);
    outputs
        .into_iter()
        .map(|format| SupportedOutput {
            format: format.clone(),
            level: "supported".to_string(),
            notes: None,
        })
        .collect()
}

#[tauri::command]
pub fn get_conversion_plan(
    input_format: String,
    output_format: String,
    state: State<'_, AppState>,
) -> Option<ConversionPlan> {
    planner::plan_conversion(&input_format, &output_format, &state.registry)
}

#[tauri::command]
pub async fn start_conversion(
    input_path: String,
    output_format: String,
    output_dir: String,
    options: ConversionOptions,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let input = std::path::Path::new(&input_path);
    let detected = detectors::detect_format(input)?;

    let job_id = Uuid::new_v4().to_string();
    let output_path = crate::filesystem::generate_output_filename(
        &detected.name,
        &output_format,
        "-converted",
        std::path::Path::new(&output_dir),
    );

    let job = ConversionJob::new(
        job_id.clone(),
        detected,
        output_format.clone(),
        output_path.to_string_lossy().to_string(),
        options,
    );

    {
        let mut queue = state.job_queue.lock().await;
        queue.add(job);
    }

    let queue = state.job_queue.clone();
    let registry = state.registry.clone();
    tokio::spawn(async move {
        worker::process_queue(queue.clone(), registry, Some(app)).await;
    });

    Ok(job_id)
}

#[tauri::command]
pub async fn get_job_status(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<ConversionJob, String> {
    let queue = state.job_queue.lock().await;
    queue
        .get(&job_id)
        .cloned()
        .ok_or_else(|| format!("Job {} not found", job_id))
}

#[tauri::command]
pub async fn cancel_job(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut queue = state.job_queue.lock().await;
    if queue.cancel(&job_id) {
        Ok(())
    } else {
        Err("Cannot cancel this job".to_string())
    }
}

#[tauri::command]
pub async fn retry_job(
    job_id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let new_job = {
        let queue = state.job_queue.lock().await;
        let job = queue
            .get(&job_id)
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        let new_id = Uuid::new_v4().to_string();
        let mut new_job = job.clone();
        new_job.id = new_id;
        new_job.status = JobStatus::Queued;
        new_job.progress = 0.0;
        new_job.error = None;
        new_job.started_at = None;
        new_job.completed_at = None;
        new_job
    };

    let new_id = new_job.id.clone();
    {
        let mut queue = state.job_queue.lock().await;
        queue.add(new_job);
    }

    let queue = state.job_queue.clone();
    let registry = state.registry.clone();
    tokio::spawn(async move {
        worker::process_queue(queue, registry, Some(app)).await;
    });

    Ok(new_id)
}

#[tauri::command]
pub async fn start_batch(
    files: Vec<serde_json::Value>,
    output_dir: String,
    options: ConversionOptions,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();

    {
        let mut queue = state.job_queue.lock().await;
        for file_entry in &files {
            let input_path = file_entry["inputPath"]
                .as_str()
                .ok_or("Missing inputPath")?;
            let output_format = file_entry["outputFormat"]
                .as_str()
                .ok_or("Missing outputFormat")?;

            let input = std::path::Path::new(input_path);
            let detected = detectors::detect_format(input)?;

            let job_id = Uuid::new_v4().to_string();
            let output_path = crate::filesystem::generate_output_filename(
                &detected.name,
                output_format,
                "-converted",
                std::path::Path::new(&output_dir),
            );

            let job = ConversionJob::new(
                job_id.clone(),
                detected,
                output_format.to_string(),
                output_path.to_string_lossy().to_string(),
                options.clone(),
            );

            queue.add(job);
            ids.push(job_id);
        }
    }

    let queue = state.job_queue.clone();
    let registry = state.registry.clone();
    tokio::spawn(async move {
        worker::process_queue(queue, registry, Some(app)).await;
    });

    Ok(ids)
}

#[tauri::command]
pub async fn get_batch_progress(
    job_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<BatchProgress, String> {
    let queue = state.job_queue.lock().await;
    let mut completed = 0usize;
    let mut failed = 0usize;
    let mut cancelled = 0usize;
    let mut current_id = None;
    let mut current_name = None;

    for id in &job_ids {
        if let Some(job) = queue.get(id) {
            match job.status {
                JobStatus::Completed => completed += 1,
                JobStatus::Failed => failed += 1,
                JobStatus::Cancelled => cancelled += 1,
                JobStatus::Converting => {
                    current_id = Some(job.id.clone());
                    current_name = Some(job.input_file.name.clone());
                }
                _ => {}
            }
        }
    }

    let total = job_ids.len();
    let overall = if total > 0 {
        (completed as f32 / total as f32) * 100.0
    } else {
        0.0
    };

    Ok(BatchProgress {
        total_jobs: total,
        completed_jobs: completed,
        failed_jobs: failed,
        cancelled_jobs: cancelled,
        current_job_id: current_id,
        current_job_name: current_name,
        overall_progress: overall,
        elapsed_ms: 0,
        estimated_remaining_ms: None,
    })
}

#[tauri::command]
pub async fn cancel_all_jobs(state: State<'_, AppState>) -> Result<(), String> {
    let mut queue = state.job_queue.lock().await;
    queue.cancel_all();
    Ok(())
}

#[tauri::command]
pub fn get_conversion_history() -> Vec<ConversionResult> {
    crate::history::load_history()
}

#[tauri::command]
pub fn clear_history() {
    crate::history::clear();
}

#[tauri::command]
pub fn get_settings() -> crate::settings::AppSettings {
    crate::settings::load_settings()
}

#[tauri::command]
pub fn save_settings(settings: crate::settings::AppSettings) -> Result<(), String> {
    crate::settings::save_settings_to_disk(&settings)
}

#[tauri::command]
pub fn get_engine_status(state: State<'_, AppState>) -> Vec<EngineInfo> {
    state.registry.engine_status()
}

#[tauri::command]
pub fn open_file_location(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if let Some(parent) = p.parent() {
        open::that(parent).map_err(|e| format!("Cannot open folder: {}", e))?;
    }
    Ok(())
}
