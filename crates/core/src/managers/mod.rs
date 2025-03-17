mod error;

use crate::{
    domain::job::{JobContext, PendingJob},
    runners::job::JobRunner,
    storage::Storage,
    workers::job::{JobWorker, JobWorkerHandle, JobWorkerSettings},
};

pub struct JobfireManager<T: JobContext> {
    context: T,
    storage: Storage,
    job_worker_handle: JobWorkerHandle,
}

impl<TJobContext: JobContext> JobfireManager<TJobContext> {
    pub fn start(
        context: TJobContext,
        storage: Storage,
        job_runner: Box<dyn JobRunner<TJobContext>>,
        job_worker_settings: JobWorkerSettings,
    ) -> error::Result<Self> {
        let job_worker_handle = JobWorker::start(
            job_worker_settings,
            storage.clone(),
            context.clone(),
            job_runner,
        );

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

    pub async fn schedule(&self, pending_job: &PendingJob) -> error::Result<()> {
        self.storage.pending_job_repo().add(pending_job).await?;
        Ok(())
    }
}
