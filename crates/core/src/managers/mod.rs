mod error;

use crate::{
    domain::{JobContext, PendingJob},
    storage::Storage,
    workers::job::{JobWorker, JobWorkerHandle, JobWorkerSettings},
};

pub struct JobfireManager<T: JobContext> {
    context: T,
    storage: Storage<T>,
    job_worker_handle: JobWorkerHandle,
}

impl<T: JobContext> JobfireManager<T> {
    pub fn start(
        context: T,
        storage: Storage<T>,
        job_worker_settings: JobWorkerSettings,
    ) -> error::Result<Self> {
        let job_worker_handle =
            JobWorker::start(job_worker_settings, storage.clone(), context.clone());

        log::info!("JobfireManager started");
        Ok(Self {
            context,
            storage,
            job_worker_handle,
        })
    }

    pub async fn stop(&self) -> error::Result<()> {
        log::info!("JobfireManager stopping");
        self.job_worker_handle
            .stop()
            .await
            .map_err(|_| error::Error::StopFailed)?;

        log::info!("JobfireManager stopped");
        Ok(())
    }

    pub async fn schedule(&self, pending_job: PendingJob<T>) -> error::Result<()> {
        self.storage.pending_job_repo().add(pending_job).await?;
        Ok(())
    }
}
