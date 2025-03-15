use std::sync::Arc;

use getset::Getters;

use crate::domain::JobContext;

mod error;
mod job;

pub use error::{Error, Result};
pub use job::{FailedJobRepo, PendingJobRepo, RunningJobRepo, SuccessfulJobRepo};

#[derive(Clone, Getters)]
#[getset(get = "pub")]
pub struct Storage<T: JobContext> {
    pending_job_repo: Arc<dyn PendingJobRepo<T>>,
    running_job_repo: Arc<dyn RunningJobRepo<T>>,
    successful_job_repo: Arc<dyn SuccessfulJobRepo<T>>,
    failed_job_repo: Arc<dyn FailedJobRepo<T>>,
}

impl<T: JobContext> Storage<T> {
    pub fn new(
        pending_job_repo: Arc<dyn PendingJobRepo<T>>,
        running_job_repo: Arc<dyn RunningJobRepo<T>>,
        successful_job_repo: Arc<dyn SuccessfulJobRepo<T>>,
        failed_job_repo: Arc<dyn FailedJobRepo<T>>,
    ) -> Self {
        Self {
            pending_job_repo,
            running_job_repo,
            successful_job_repo,
            failed_job_repo,
        }
    }
}
