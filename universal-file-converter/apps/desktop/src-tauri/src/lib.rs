pub mod commands;
pub mod conversion;
pub mod converters;
pub mod detectors;
pub mod engines;
pub mod filesystem;
pub mod history;
pub mod jobs;
pub mod planner;
pub mod process;
pub mod security;
pub mod settings;
pub mod validators;

use commands::{
    cancel_all_jobs, cancel_job, clear_history, detect_file, get_batch_progress,
    get_conversion_history, get_conversion_plan, get_engine_status, get_job_status,
    get_settings, get_supported_outputs, open_file_location, retry_job, save_settings,
    start_batch, start_conversion,
};
use engines::registry::ConverterRegistry;
use jobs::queue::JobQueue;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub registry: Arc<ConverterRegistry>,
    pub job_queue: Arc<Mutex<JobQueue>>,
}

pub fn run() {
    env_logger::init();

    let temp_base = filesystem::get_temp_base();
    filesystem::cleanup_abandoned_jobs(&temp_base, 1);

    let registry = Arc::new(ConverterRegistry::new());
    let job_queue = Arc::new(Mutex::new(JobQueue::new()));

    let state = AppState {
        registry,
        job_queue,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            detect_file,
            get_supported_outputs,
            start_conversion,
            get_job_status,
            cancel_job,
            retry_job,
            start_batch,
            get_batch_progress,
            cancel_all_jobs,
            get_conversion_history,
            clear_history,
            get_settings,
            save_settings,
            get_engine_status,
            open_file_location,
            get_conversion_plan,
        ])
        .run(tauri::generate_context!())
        .expect("error running application");
}
