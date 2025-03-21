mod failed_job;
mod job;
mod pending_job;
mod running_job;
mod successful_job;

use failed_job::FailedJobRepoImpl;
use job::JobRepoImpl;
use jobfire_core::{
    builders::jobfire_manager::JobfireManagerBuilder, domain::job::context::JobContextData,
    storage::Storage,
};
use pending_job::PendingJobRepoImpl;
use running_job::RunningJobRepoImpl;
use successful_job::SuccessfulJobRepoImpl;

#[derive(Default)]
pub struct InMemoryStorage {
    job_repo: JobRepoImpl,
    pending_job_repo: PendingJobRepoImpl,
    running_job_repo: RunningJobRepoImpl,
    successful_job_repo: SuccessfulJobRepoImpl,
    failed_job_repo: FailedJobRepoImpl,
}

impl From<InMemoryStorage> for Storage {
    fn from(value: InMemoryStorage) -> Self {
        let job_repo = Box::new(value.job_repo);
        let pending_job_repo = Box::new(value.pending_job_repo);
        let running_job_repo = Box::new(value.running_job_repo);
        let successful_job_repo = Box::new(value.successful_job_repo);
        let failed_job_repo = Box::new(value.failed_job_repo);
        Self::new(
            job_repo,
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
