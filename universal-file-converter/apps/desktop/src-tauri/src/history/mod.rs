use crate::conversion::result::ConversionResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct HistoryStore {
    entries: Vec<ConversionResult>,
}

fn history_path() -> PathBuf {
    dirs_next::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("UniversalFileConverter")
        .join("history.json")
}

pub fn load_history() -> Vec<ConversionResult> {
    let path = history_path();
    if !path.exists() {
        return Vec::new();
    }
    let Ok(data) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(store) = serde_json::from_str::<HistoryStore>(&data) else {
        return Vec::new();
    };
    store.entries
}

pub fn save_history(entries: &[ConversionResult]) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let store = HistoryStore {
        entries: entries.to_vec(),
    };
    let _ = fs::write(&path, serde_json::to_string_pretty(&store).unwrap_or_default());
}

pub fn add_to_history(result: ConversionResult) {
    let mut history = load_history();
    history.insert(0, result);
    if history.len() > 500 {
        history.truncate(500);
    }
    save_history(&history);
}

pub fn clear() {
    save_history(&[]);
}
