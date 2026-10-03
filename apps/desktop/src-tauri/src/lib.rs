//! DocSnap Tauri backend.
//!
//! Modules track the Phase 1 architecture. Most are stubs now and are filled in
//! phase by phase. The IPC surface lives in `commands` and must remain the only
//! way the frontend reaches native code.

mod commands;
mod camera;
mod document_detection;
mod export;
mod filesystem;
mod image_processing;
mod ocr;
mod pdf;
mod perspective;
mod projects;
mod security;
mod settings;

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub offline: bool,
    pub ocr_engine: &'static str,
}

/// Smoke-test command. Confirms the IPC bridge is live during Phase 2.
#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        name: "DocSnap",
        version: env!("CARGO_PKG_VERSION"),
        offline: true,
        ocr_engine: "tesseract-5-sidecar (not yet wired)",
    }
}

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init()
        .ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![get_app_info])
        .run(tauri::generate_context!())
        .expect("DocSnap failed to start");
}
