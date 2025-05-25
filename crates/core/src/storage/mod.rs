pub mod error;
pub mod job;
pub mod memory;
pub mod run;

use getset::Getters;
use job::{JobRepo, PendingJobRepo, RunningJobRepo};
use run::{FailedRunRepo, SuccessfulRunRepo};
use std::sync::Arc;

use crate::services::{verify::VerifyService, Services};

#[derive(Clone, Getters)]
#[getset(get = "pub")]
pub struct Storage {
    inner: Arc<StorageInner>,
}

impl VerifyService for Storage {
    fn verify(
        &self,
        _services: &crate::services::Services,
    ) -> Result<(), crate::services::verify::ServiceMissing> {
        Ok(())
    }
}

struct StorageInner {
    job_repo: Box<dyn JobRepo>,
    pending_job_repo: Box<dyn PendingJobRepo>,
    running_job_repo: Box<dyn RunningJobRepo>,
    successful_run_repo: Box<dyn SuccessfulRunRepo>,
    failed_run_repo: Box<dyn FailedRunRepo>,
}

impl Storage {
    pub fn new(
        job_repo: Box<dyn JobRepo>,
        pending_job_repo: Box<dyn PendingJobRepo>,
        running_job_repo: Box<dyn RunningJobRepo>,
        successful_run_repo: Box<dyn SuccessfulRunRepo>,
        failed_run_repo: Box<dyn FailedRunRepo>,
    ) -> Self {
        Self {
            inner: Arc::new(StorageInner {
                job_repo,
                pending_job_repo,
                running_job_repo,
                successful_run_repo,
                failed_run_repo,
            }),
        }
    }

    pub fn job_repo(&self) -> &dyn JobRepo {
        self.inner.job_repo.as_ref()
    }

    pub fn pending_job_repo(&self) -> &dyn PendingJobRepo {
        self.inner.pending_job_repo.as_ref()
    }

    pub fn running_job_repo(&self) -> &dyn RunningJobRepo {
        self.inner.running_job_repo.as_ref()
    }

    pub fn successful_run_repo(&self) -> &dyn SuccessfulRunRepo {
        self.inner.successful_run_repo.as_ref()
    }

    pub fn failed_run_repo(&self) -> &dyn FailedRunRepo {
        self.inner.failed_run_repo.as_ref()
    }
}

pub trait AddStorageService {
    fn add_storage(&self, storage: impl Into<Storage>) -> Self;
}

impl AddStorageService for Services {
    fn add_storage(&self, storage: impl Into<Storage>) -> Self {
        self.add_service(storage.into());
        self.clone()
    }
}
