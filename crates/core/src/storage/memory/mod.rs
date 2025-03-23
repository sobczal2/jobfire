use job::{MemoryJobRepo, pending::MemoryPendingJobRepo, running::MemoryRunningJobRepo};
use run::{failed::MemoryFailedRunRepo, successful::MemorySuccessfulRunRepo};

use crate::{builders::job_manager::JobManagerBuilder, domain::job::context::ContextData};

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

pub trait WithMemoryStorage {
    fn with_memory_storage(&self) -> Self;
}

impl<TData: ContextData> WithMemoryStorage for JobManagerBuilder<TData> {
    fn with_memory_storage(&self) -> Self {
        self.with_storage(MemoryStorage::default())
    }
}
