pub mod error;
pub mod job;

use getset::Getters;
use job::{FailedJobRepo, PendingJobRepo, RunningJobRepo, SuccessfulJobRepo};
use std::sync::Arc;

#[derive(Clone, Getters)]
#[getset(get = "pub")]
pub struct Storage {
    pending_job_repo: Arc<dyn PendingJobRepo>,
    running_job_repo: Arc<dyn RunningJobRepo>,
    successful_job_repo: Arc<dyn SuccessfulJobRepo>,
    failed_job_repo: Arc<dyn FailedJobRepo>,
}

impl Storage {
    pub fn new(
        pending_job_repo: Arc<dyn PendingJobRepo>,
        running_job_repo: Arc<dyn RunningJobRepo>,
        successful_job_repo: Arc<dyn SuccessfulJobRepo>,
        failed_job_repo: Arc<dyn FailedJobRepo>,
    ) -> Self {
        Self {
            pending_job_repo,
            running_job_repo,
            successful_job_repo,
            failed_job_repo,
        }
    }
}
