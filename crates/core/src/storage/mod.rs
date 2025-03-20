pub mod error;
pub mod job;

use getset::Getters;
use job::{FailedJobRepo, PendingJobRepo, RunningJobRepo, SuccessfulJobRepo};
use std::sync::Arc;

#[derive(Clone, Getters)]
#[getset(get = "pub")]
pub struct Storage {
    inner: Arc<StorageInner>,
}

struct StorageInner {
    pending_job_repo: Box<dyn PendingJobRepo>,
    running_job_repo: Box<dyn RunningJobRepo>,
    successful_job_repo: Box<dyn SuccessfulJobRepo>,
    failed_job_repo: Box<dyn FailedJobRepo>,
}

impl Storage {
    pub fn new(
        pending_job_repo: Box<dyn PendingJobRepo>,
        running_job_repo: Box<dyn RunningJobRepo>,
        successful_job_repo: Box<dyn SuccessfulJobRepo>,
        failed_job_repo: Box<dyn FailedJobRepo>,
    ) -> Self {
        Self {
            inner: Arc::new(StorageInner {
                pending_job_repo,
                running_job_repo,
                successful_job_repo,
                failed_job_repo,
            }),
        }
    }

    pub fn pending_job_repo(&self) -> &dyn PendingJobRepo {
        self.inner.pending_job_repo.as_ref()
    }

    pub fn running_job_repo(&self) -> &dyn RunningJobRepo {
        self.inner.running_job_repo.as_ref()
    }

    pub fn successful_job_repo(&self) -> &dyn SuccessfulJobRepo {
        self.inner.successful_job_repo.as_ref()
    }

    pub fn failed_job_repo(&self) -> &dyn FailedJobRepo {
        self.inner.failed_job_repo.as_ref()
    }
}
