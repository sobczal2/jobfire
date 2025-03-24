use job::{MemoryJobRepo, pending::MemoryPendingJobRepo, running::MemoryRunningJobRepo};
use run::{failed::MemoryFailedRunRepo, successful::MemorySuccessfulRunRepo};

use super::Storage;

pub mod job;
pub mod run;

#[derive(Default)]
pub struct MemoryStorage {
    job_repo: MemoryJobRepo,
    pending_job_repo: MemoryPendingJobRepo,
    running_job_repo: MemoryRunningJobRepo,
    successful_run_repo: MemorySuccessfulRunRepo,
    failed_run_repo: MemoryFailedRunRepo,
}

impl From<MemoryStorage> for Storage {
    fn from(value: MemoryStorage) -> Self {
        Storage::new(
            Box::new(value.job_repo),
            Box::new(value.pending_job_repo),
            Box::new(value.running_job_repo),
            Box::new(value.successful_run_repo),
            Box::new(value.failed_run_repo),
        )
    }
}
