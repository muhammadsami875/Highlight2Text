use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub fn create_job_dir(base_temp: &Path) -> Result<PathBuf, String> {
    let job_id = Uuid::new_v4().to_string();
    let job_dir = base_temp.join("jobs").join(&job_id);
    fs::create_dir_all(job_dir.join("input"))
        .map_err(|e| format!("Cannot create job input dir: {}", e))?;
    fs::create_dir_all(job_dir.join("intermediate"))
        .map_err(|e| format!("Cannot create job intermediate dir: {}", e))?;
    fs::create_dir_all(job_dir.join("output"))
        .map_err(|e| format!("Cannot create job output dir: {}", e))?;
    Ok(job_dir)
}

pub fn cleanup_job_dir(job_dir: &Path) {
    let _ = fs::remove_dir_all(job_dir);
}

pub fn generate_output_filename(
    source_name: &str,
    output_format: &str,
    suffix: &str,
    output_dir: &Path,
) -> PathBuf {
    let raw_stem = Path::new(source_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let stem = crate::security::sanitize_filename(raw_stem);
    let stem = if stem.is_empty() { "output".to_string() } else { stem };

    let base_name = format!("{}{}.{}", stem, suffix, output_format);
    let mut output_path = output_dir.join(&base_name);

    let mut counter = 1u32;
    while output_path.exists() {
        let numbered = format!("{}{}-{}.{}", stem, suffix, counter, output_format);
        output_path = output_dir.join(&numbered);
        counter += 1;
        if counter > 9999 {
            break;
        }
    }

    output_path
}

pub fn get_temp_base() -> PathBuf {
    let base = dirs_next::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("UniversalFileConverter")
        .join("temp");
    let _ = fs::create_dir_all(&base);
    base
}

pub fn cleanup_abandoned_jobs(base_temp: &Path, max_age_hours: u64) {
    let jobs_dir = base_temp.join("jobs");
    let Ok(entries) = fs::read_dir(&jobs_dir) else {
        return;
    };

    let cutoff = std::time::SystemTime::now()
        - std::time::Duration::from_secs(max_age_hours * 3600);

    for entry in entries.flatten() {
        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                if modified < cutoff {
                    let _ = fs::remove_dir_all(entry.path());
                }
            }
        }
    }
}
