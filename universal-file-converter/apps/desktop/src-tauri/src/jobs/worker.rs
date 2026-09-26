use crate::conversion::error::ConversionError;
use crate::conversion::job::JobStatus;
use crate::conversion::request::ConversionRequest;
use crate::engines::registry::ConverterRegistry;
use crate::jobs::queue::JobQueue;
use crate::planner;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn process_queue(
    queue: Arc<Mutex<JobQueue>>,
    registry: Arc<ConverterRegistry>,
) {
    loop {
        let job_id = {
            let q = queue.lock().await;
            q.next_pending().map(|s| s.to_string())
        };

        let job_id = match job_id {
            Some(id) => id,
            None => break,
        };

        process_single_job(&job_id, &queue, &registry).await;
    }
}

async fn process_single_job(
    job_id: &str,
    queue: &Arc<Mutex<JobQueue>>,
    registry: &Arc<ConverterRegistry>,
) {
    {
        let mut q = queue.lock().await;
        if let Some(job) = q.get_mut(job_id) {
            job.set_status(JobStatus::Planning);
            job.progress = 5.0;
            job.progress_message = "Planning conversion...".into();
        }
    }

    let (from, to, input_path, output_dir, options) = {
        let q = queue.lock().await;
        let job = match q.get(job_id) {
            Some(j) => j,
            None => return,
        };
        (
            job.input_file.detected_format.clone(),
            job.output_format.clone(),
            PathBuf::from(&job.input_file.path),
            PathBuf::from(&job.output_path)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .to_path_buf(),
            job.options.clone(),
        )
    };

    let plan = planner::plan_conversion(&from, &to, registry);
    let plan = match plan {
        Some(p) => p,
        None => {
            let mut q = queue.lock().await;
            if let Some(job) = q.get_mut(job_id) {
                job.fail(ConversionError::unsupported(&from, &to));
            }
            return;
        }
    };

    {
        let mut q = queue.lock().await;
        if let Some(job) = q.get_mut(job_id) {
            if job.status == JobStatus::Cancelled {
                return;
            }
            job.plan = Some(plan.clone());
            job.set_status(JobStatus::Converting);
            job.progress = 10.0;
            job.progress_message = "Converting...".into();
        }
    }

    let job_dir = std::env::temp_dir()
        .join("ufc_jobs")
        .join(job_id);
    let _ = std::fs::create_dir_all(job_dir.join("output"));

    let total_steps = plan.steps.len();
    let mut current_input = input_path;
    let mut current_format = from;

    for (step_idx, step) in plan.steps.iter().enumerate() {
        {
            let q = queue.lock().await;
            if let Some(job) = q.get(job_id) {
                if job.status == JobStatus::Cancelled {
                    return;
                }
            }
        }

        let request = ConversionRequest {
            input_path: current_input.clone(),
            detected_format: current_format.clone(),
            output_format: step.to.clone(),
            output_dir: if step_idx == total_steps - 1 {
                output_dir.clone()
            } else {
                job_dir.join("intermediate")
            },
            options: options.clone(),
        };

        let step_progress = 10.0 + (80.0 * (step_idx as f32 + 0.5) / total_steps as f32);
        {
            let mut q = queue.lock().await;
            if let Some(job) = q.get_mut(job_id) {
                job.progress = step_progress;
                job.progress_message = format!(
                    "Step {}/{}: {} -> {}",
                    step_idx + 1,
                    total_steps,
                    step.from,
                    step.to
                );
            }
        }

        let step_from = step.from.clone();
        let step_to = step.to.clone();
        let reg = Arc::clone(registry);
        let jd = job_dir.clone();
        let req = request.clone();

        let result = tokio::task::spawn_blocking(move || {
            match reg.best_converter(&step_from, &step_to) {
                Some(converter) => converter.convert(&req, &jd),
                None => Err(ConversionError::unsupported(&step_from, &step_to)),
            }
        });

        match result.await {
            Ok(Ok(output_path)) => {
                current_input = output_path;
                current_format = step.to.clone();
            }
            Ok(Err(err)) => {
                let mut q = queue.lock().await;
                if let Some(job) = q.get_mut(job_id) {
                    job.fail(err);
                }
                return;
            }
            Err(e) => {
                let mut q = queue.lock().await;
                if let Some(job) = q.get_mut(job_id) {
                    job.fail(ConversionError::engine_failure(
                        "worker",
                        &format!("Task panicked: {}", e),
                    ));
                }
                return;
            }
        }
    }

    {
        let mut q = queue.lock().await;
        if let Some(job) = q.get_mut(job_id) {
            job.set_status(JobStatus::Validating);
            job.progress = 95.0;
            job.progress_message = "Validating output...".into();
        }
    }

    if !current_input.exists() {
        let mut q = queue.lock().await;
        if let Some(job) = q.get_mut(job_id) {
            job.fail(ConversionError::validation_failed(
                "Output file was not created",
            ));
        }
        return;
    }

    let final_output = output_dir.join(
        current_input
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("output")),
    );

    if current_input != final_output {
        let _ = std::fs::create_dir_all(&output_dir);
        if let Err(e) = std::fs::copy(&current_input, &final_output) {
            let mut q = queue.lock().await;
            if let Some(job) = q.get_mut(job_id) {
                job.fail(ConversionError::io_error(&format!(
                    "Cannot copy output: {}",
                    e
                )));
            }
            return;
        }
    }

    {
        let mut q = queue.lock().await;
        if let Some(job) = q.get_mut(job_id) {
            job.complete(final_output.to_string_lossy().to_string());
        }
    }

    let _ = std::fs::remove_dir_all(&job_dir);
}
