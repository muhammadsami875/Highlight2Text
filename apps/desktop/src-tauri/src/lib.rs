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
use specta::Type;
use tauri_specta::{collect_commands, Builder};

#[derive(Serialize, Type, Clone)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub offline: bool,
    pub ocr_engine: &'static str,
}

#[tauri::command]
#[specta::specta]
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

    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        get_app_info,
        commands::import_image,
        commands::detect_document_boundary,
        commands::apply_perspective,
        commands::apply_enhancement,
    ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/types/bindings.ts",
        )
        .ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("DocSnap failed to start");
}
