use crate::{
    builders::{Builder, job_manager::JobManagerBuilder, job_scheduler::JobSchedulerBuilder},
    domain::job::{
        Job,
        context::{JobContext, JobContextData},
        id::JobId,
        r#impl::JobImpl,
        scheduler::{self, JobScheduler},
    },
    runners::job::JobRunner,
    storage::{self, Storage},
    util::r#async::poll_predicate,
    workers::job::{JobWorker, JobWorkerHandle, JobWorkerSettings, State},
};
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use thiserror::Error;
use tokio::time::interval;

#[derive(Debug, Error)]
pub enum Error {
    #[error("stop failed")]
    StopFailed,
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
    #[error("scheduler error: {0}")]
    Scheduler(#[from] scheduler::Error),
    #[error("failed to build a job")]
    JobBuildFailed,
}

pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub struct JobManager<TData: JobContextData> {
    context: JobContext<TData>,
    storage: Storage,
    job_runner: JobRunner<TData>,
    job_worker_settings: JobWorkerSettings,
    job_worker_handle: JobWorkerHandle,
    job_scheduler: Arc<dyn JobScheduler>,
}

impl<TData: JobContextData> JobManager<TData> {
    pub fn start(
        context: JobContext<TData>,
        storage: Storage,
        job_runner: JobRunner<TData>,
        job_worker_settings: JobWorkerSettings,
        job_scheduler: Arc<dyn JobScheduler>,
    ) -> Self {
        let job_worker = JobWorker::new(job_worker_settings, storage.clone(), job_runner.clone());
        let job_worker_handle = job_worker.start();

        log::info!("JobfireManager started");
        Self {
            context,
            storage,
            job_runner,
            job_worker_settings,
            job_worker_handle,
            job_scheduler,
        }
    }

    pub async fn stop(self) -> Result<()> {
        log::info!("JobfireManager stopping");
        self.job_worker_handle
            .stop()
            .await
            .map_err(|_| Error::StopFailed)?;

        let job_worker_handle = self.job_worker_handle.clone();

        poll_predicate(
            async move || job_worker_handle.get_state().await == State::Stopped,
            interval(Duration::milliseconds(100).to_std().unwrap()),
        )
        .await;

        log::info!("JobfireManager stopped");
        Ok(())
    }

    pub fn basic_builder(context_data: TData) -> JobManagerBuilder<TData> {
        let builder = JobManagerBuilder::default();
        builder.with_job_worker_settings(JobWorkerSettings::default());
        builder.with_context_data(context_data);
        builder.with_job_scheduler_factory(Box::new(|storage| {
            let builder = JobSchedulerBuilder::default();
            builder.with_storage(storage);
            builder.build().unwrap()
        }));
        builder
    }

    pub fn builder() -> JobManagerBuilder<TData> {
        JobManagerBuilder::default()
    }
}

impl<TData: JobContextData> JobManager<TData> {
    pub async fn schedule(
        &self,
        job_impl: impl JobImpl<TData>,
        at: DateTime<Utc>,
    ) -> Result<JobId> {
        let job = Job::from_impl(job_impl).map_err(|_| Error::JobBuildFailed)?;
        let job_id = *job.id();
        self.job_scheduler.schedule(job, at).await?;
        Ok(job_id)
    }

    pub async fn cancel(&self, job_id: &JobId) -> Result<()> {
        self.job_scheduler.cancel(job_id).await?;
        Ok(())
    }

    pub async fn reschedule(
        &self,
        job_id: &JobId,
        new_scheduled_at: DateTime<chrono::Utc>,
    ) -> Result<()> {
        self.job_scheduler
            .reschedule(job_id, new_scheduled_at)
            .await?;
        Ok(())
    }
}
