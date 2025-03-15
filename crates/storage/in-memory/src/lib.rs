mod job;

use std::sync::Arc;

use job::{
    failed::FailedJobRepoImpl, pending::PendingJobRepoImpl, running::RunningJobRepoImpl,
    successful::SuccessfulJobRepoImpl,
};
use jobfire_core::prelude::*;

pub struct InMemoryStorage<T: JobContext> {
    pending_job_repo: PendingJobRepoImpl<T>,
    running_job_repo: RunningJobRepoImpl<T>,
    successful_job_repo: SuccessfulJobRepoImpl<T>,
    failed_job_repo: FailedJobRepoImpl<T>,
}

impl<T: JobContext> Default for InMemoryStorage<T> {
    fn default() -> Self {
        Self {
            pending_job_repo: Default::default(),
            running_job_repo: Default::default(),
            successful_job_repo: Default::default(),
            failed_job_repo: Default::default(),
        }
    }
}

impl<T: JobContext> From<InMemoryStorage<T>> for Storage<T> {
    fn from(value: InMemoryStorage<T>) -> Self {
        let pending_job_repo = Arc::new(value.pending_job_repo);
        let running_job_repo = Arc::new(value.running_job_repo);
        let successful_job_repo = Arc::new(value.successful_job_repo);
        let failed_job_repo = Arc::new(value.failed_job_repo);
        Self::new(
            pending_job_repo,
            running_job_repo,
            successful_job_repo,
            failed_job_repo,
        )
    }
}
