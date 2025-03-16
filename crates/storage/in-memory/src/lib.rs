mod job;

use std::sync::Arc;

use job::{
    failed::FailedJobRepoImpl, pending::PendingJobRepoImpl, running::RunningJobRepoImpl,
    successful::SuccessfulJobRepoImpl,
};
use jobfire_core::storage::Storage;

pub struct InMemoryStorage {
    pending_job_repo: PendingJobRepoImpl,
    running_job_repo: RunningJobRepoImpl,
    successful_job_repo: SuccessfulJobRepoImpl,
    failed_job_repo: FailedJobRepoImpl,
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self {
            pending_job_repo: Default::default(),
            running_job_repo: Default::default(),
            successful_job_repo: Default::default(),
            failed_job_repo: Default::default(),
        }
    }
}

impl From<InMemoryStorage> for Storage {
    fn from(value: InMemoryStorage) -> Self {
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
