use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use thiserror::Error;
use tokio::time::interval;

use crate::{
    builders::{
        Builder, job_actions_registry::JobActionsRegistryBuilder,
        job_scheduler::JobSchedulerBuilder, jobfire_manager::JobfireManagerBuilder,
    },
    domain::job::{
        context::{JobContext, JobContextData},
        scheduler::JobScheduler,
    },
    runners::job::JobRunner,
    storage::{self, Storage},
    util::r#async::poll_predicate,
    workers::job::{JobWorker, JobWorkerHandle, JobWorkerSettings, State},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("stop failed")]
    StopFailed,
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct JobfireManager<TData: JobContextData> {
    context: JobContext<TData>,
    storage: Storage,
    job_runner: JobRunner<TData>,
    job_worker_settings: JobWorkerSettings,
    job_worker_handle: JobWorkerHandle,
    job_scheduler: Arc<dyn JobScheduler>,
}

impl<TData: JobContextData> JobfireManager<TData> {
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

    pub fn builder(context_data: TData) -> JobfireManagerBuilder<TData> {
        let builder = JobfireManagerBuilder::default();
        builder.with_job_worker_settings(JobWorkerSettings::default());
        builder.with_context_data(context_data);
        builder.with_job_scheduler_factory(Box::new(|storage| {
            let builder = JobSchedulerBuilder::default();
            builder.with_storage(storage);
            builder.build().unwrap()
        }));
        builder
    }
}

#[async_trait]
impl<TData: JobContextData> JobScheduler for JobfireManager<TData> {
    async fn schedule(
        &self,
        pending_job: &crate::domain::job::pending::PendingJob,
    ) -> crate::domain::job::scheduler::Result<()> {
        self.job_scheduler.schedule(pending_job).await
    }

    async fn cancel(
        &self,
        job_id: &crate::domain::job::id::JobId,
    ) -> crate::domain::job::scheduler::Result<()> {
        self.job_scheduler.cancel(job_id).await
    }

    async fn reschedule(
        &self,
        job_id: &crate::domain::job::id::JobId,
        new_scheduled_at: chrono::DateTime<chrono::Utc>,
    ) -> crate::domain::job::scheduler::Result<()> {
        self.job_scheduler
            .reschedule(job_id, new_scheduled_at)
            .await
    }
}
