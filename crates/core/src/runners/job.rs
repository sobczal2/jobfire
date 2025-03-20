use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use thiserror::Error;

use crate::{
    domain::job::{
        context::{JobContext, JobContextData},
        id::JobId,
        pending::PendingJob,
        running::RunningJob,
    },
    registries::job_actions::JobActionsRegistry,
    storage::{self, Storage},
};

use super::{
    on_fail::{OnFailRunner, OnFailRunnerInput},
    on_success::{OnSuccessRunner, OnSuccessRunnerInput},
};

#[derive(Error, Debug)]
enum Error {
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
    #[error("job actions not found")]
    JobActionsNotFound,
}

type Result<T> = std::result::Result<T, Error>;

pub struct JobRunnerInput {
    pending_job: PendingJob,
}

impl JobRunnerInput {
    pub fn new(pending_job: PendingJob) -> Self {
        Self { pending_job }
    }
}

pub struct JobRunner<TData: JobContextData> {
    inner: Arc<JobRunnerInner<TData>>,
}

impl<TData: JobContextData> Clone for JobRunner<TData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub struct JobRunnerInner<TData: JobContextData> {
    storage: Storage,
    context: JobContext<TData>,
    job_actions_registry: JobActionsRegistry<TData>,
    on_success_runner: OnSuccessRunner<TData>,
    on_fail_runner: OnFailRunner<TData>,
}

impl<TData: JobContextData> JobRunner<TData> {
    pub fn new(
        storage: Storage,
        context: JobContext<TData>,
        job_actions_registry: JobActionsRegistry<TData>,
    ) -> Self {
        Self {
            inner: Arc::new(JobRunnerInner {
                storage: storage.clone(),
                context: context.clone(),
                job_actions_registry: job_actions_registry.clone(),
                on_success_runner: OnSuccessRunner::new(
                    storage.clone(),
                    context.clone(),
                    job_actions_registry.clone(),
                ),
                on_fail_runner: OnFailRunner::new(
                    storage.clone(),
                    context.clone(),
                    job_actions_registry.clone(),
                ),
            }),
        }
    }

    pub async fn run(&self, input: &JobRunnerInput) {
        if let Err(error) = self.run_internal(input).await {
            log::error!("error during job run: {error}");
        }
    }

    async fn run_internal(&self, input: &JobRunnerInput) -> Result<()> {
        let pending_job = &input.pending_job;
        self.save_running_job(pending_job).await?;

        let job_actions = self
            .inner
            .job_actions_registry
            .get(pending_job.impl_name())
            .ok_or(Error::JobActionsNotFound)?;

        let run_result = job_actions
            .run(pending_job, self.inner.context.clone())
            .await;

        let running_job = self.remove_running_job(pending_job.id()).await?;

        match run_result {
            Ok(report) => {
                let input = OnSuccessRunnerInput::new(pending_job.clone());
                self.inner.on_success_runner.run(&input).await;
            }
            Err(error) => {
                let input = OnFailRunnerInput::new(pending_job.clone());
                self.inner.on_fail_runner.run(&input).await;
            }
        }
        Ok(())
    }

    async fn save_running_job(&self, pending_job: &PendingJob) -> Result<()> {
        let running_job = RunningJob::new(*pending_job.id(), *pending_job.created_at(), Utc::now());
        self.inner
            .storage
            .running_job_repo()
            .add(&running_job)
            .await?;
        Ok(())
    }

    async fn remove_running_job(&self, job_id: &JobId) -> Result<RunningJob> {
        let running_job = self.inner.storage.running_job_repo().pop(job_id).await?;
        Ok(running_job)
    }
}
