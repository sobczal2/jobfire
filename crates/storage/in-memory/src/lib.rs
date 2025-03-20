mod job;

use job::{
    failed::FailedJobRepoImpl, pending::PendingJobRepoImpl, running::RunningJobRepoImpl,
    successful::SuccessfulJobRepoImpl,
};
use jobfire_core::{
    builders::jobfire_manager::JobfireManagerBuilder, domain::job::context::JobContextData,
    storage::Storage,
};

#[derive(Default)]
pub struct InMemoryStorage {
    pending_job_repo: PendingJobRepoImpl,
    running_job_repo: RunningJobRepoImpl,
    successful_job_repo: SuccessfulJobRepoImpl,
    failed_job_repo: FailedJobRepoImpl,
}

impl From<InMemoryStorage> for Storage {
    fn from(value: InMemoryStorage) -> Self {
        let pending_job_repo = Box::new(value.pending_job_repo);
        let running_job_repo = Box::new(value.running_job_repo);
        let successful_job_repo = Box::new(value.successful_job_repo);
        let failed_job_repo = Box::new(value.failed_job_repo);
        Self::new(
            pending_job_repo,
            running_job_repo,
            successful_job_repo,
            failed_job_repo,
        )
    }
}

pub trait WithInMemoryStorage {
    fn with_in_memory_storage(&self) -> Self;
}

impl<TData: JobContextData> WithInMemoryStorage for JobfireManagerBuilder<TData> {
    fn with_in_memory_storage(&self) -> Self {
        self.with_storage(InMemoryStorage::default())
    }
}
