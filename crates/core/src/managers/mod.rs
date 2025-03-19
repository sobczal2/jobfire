pub mod job_scheduler;

use std::sync::Arc;

use thiserror::Error;

use crate::{
    domain::job::{
        context::{JobContext, JobContextData},
        pending::PendingJob,
    },
    runners::job::JobRunner,
    storage::{self, Storage},
    workers::job::{JobWorker, JobWorkerHandle, JobWorkerSettings},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("stop failed")]
    StopFailed,
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct JobfireManager {
    storage: Storage,
    job_worker_handle: JobWorkerHandle,
}

impl JobfireManager {
    pub fn start<TData: JobContextData>(
        context: JobContext<TData>,
        storage: Storage,
        job_runner: Arc<dyn JobRunner<TData>>,
        job_worker_settings: JobWorkerSettings,
    ) -> Result<Self> {
        let job_worker_handle =
            JobWorker::start(job_worker_settings, storage.clone(), context, job_runner);

        log::info!("JobfireManager started");
        Ok(Self {
            storage,
            job_worker_handle,
        })
    }

    pub async fn stop(&self) -> Result<()> {
        log::info!("JobfireManager stopping");
        self.job_worker_handle
            .stop()
            .await
            .map_err(|_| Error::StopFailed)?;

        log::info!("JobfireManager stopped");
        Ok(())
    }

    pub async fn schedule(&self, pending_job: &PendingJob) -> Result<()> {
        self.storage.pending_job_repo().add(pending_job).await?;
        Ok(())
    }
}
