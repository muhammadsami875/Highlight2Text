use crate::conversion::job::{ConversionJob, JobStatus};
use std::collections::HashMap;

pub struct JobQueue {
    jobs: HashMap<String, ConversionJob>,
    order: Vec<String>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn add(&mut self, job: ConversionJob) {
        let id = job.id.clone();
        self.jobs.insert(id.clone(), job);
        self.order.push(id);
    }

    pub fn get(&self, id: &str) -> Option<&ConversionJob> {
        self.jobs.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut ConversionJob> {
        self.jobs.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<ConversionJob> {
        self.order.retain(|i| i != id);
        self.jobs.remove(id)
    }

    pub fn cancel(&mut self, id: &str) -> bool {
        if let Some(job) = self.jobs.get_mut(id) {
            if job.status == JobStatus::Queued || job.status == JobStatus::Converting {
                job.set_status(JobStatus::Cancelled);
                return true;
            }
        }
        false
    }

    pub fn cancel_all(&mut self) {
        for job in self.jobs.values_mut() {
            if job.status == JobStatus::Queued || job.status == JobStatus::Converting {
                job.set_status(JobStatus::Cancelled);
            }
        }
    }

    pub fn next_pending(&self) -> Option<&str> {
        self.order
            .iter()
            .find(|id| {
                self.jobs
                    .get(id.as_str())
                    .map(|j| j.status == JobStatus::Queued)
                    .unwrap_or(false)
            })
            .map(|s| s.as_str())
    }

    pub fn completed_count(&self) -> usize {
        self.jobs
            .values()
            .filter(|j| j.status == JobStatus::Completed)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.jobs
            .values()
            .filter(|j| j.status == JobStatus::Failed)
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.jobs.len()
    }

    pub fn all_jobs(&self) -> Vec<&ConversionJob> {
        self.order
            .iter()
            .filter_map(|id| self.jobs.get(id))
            .collect()
    }
}
