use chrono::Utc;
use thiserror::Error;

use crate::{
    domain::job::{
        self, Job,
        context::{JobContext, JobContextData},
        failed::FailedJob,
        id::JobId,
        pending::PendingJob,
        running::RunningJob,
    },
    registries::job_actions::JobActionsRegistry,
    storage::{self, Storage},
};

#[derive(Error, Debug)]
enum Error {
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
    #[error("job actions not found")]
    JobActionsNotFound,
    #[error("on_fail callback failed: {0}")]
    CallbackFailed(#[from] job::error::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct OnFailRunnerInput {
    job: Job,
    pending_job: PendingJob,
    running_job: RunningJob,
    error: job::error::Error,
}

impl OnFailRunnerInput {
    pub fn new(
        job: Job,
        pending_job: PendingJob,
        running_job: RunningJob,
        error: job::error::Error,
    ) -> Self {
        Self {
            job,
            pending_job,
            running_job,
            error,
        }
    }
}

pub struct OnFailRunner<TData: JobContextData> {
    storage: Storage,
    context: JobContext<TData>,
    job_actions_registry: JobActionsRegistry<TData>,
}

impl<TData: JobContextData> Clone for OnFailRunner<TData> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            context: self.context.clone(),
            job_actions_registry: self.job_actions_registry.clone(),
        }
    }
}

impl<TData: JobContextData> OnFailRunner<TData> {
    pub fn new(
        storage: Storage,
        context: JobContext<TData>,
        job_actions_registry: JobActionsRegistry<TData>,
    ) -> Self {
        Self {
            storage,
            context,
            job_actions_registry,
        }
    }

    pub async fn run(&self, input: &OnFailRunnerInput) {
        if let Err(error) = self.run_internal(input).await {
            log::error!("error during on_fail callback run: {error}");
        }
    }

    async fn run_internal(&self, input: &OnFailRunnerInput) -> Result<()> {
        let failed_job = FailedJob::new(
            *input.job.id(),
            *input.pending_job.scheduled_at(),
            Utc::now(),
            input.error.clone(),
        );

        self.storage.failed_job_repo().add(&failed_job).await?;

        let job_actions = self
            .job_actions_registry
            .get(input.job.r#impl().name())
            .ok_or(Error::JobActionsNotFound)?;

        job_actions
            .on_fail(input.job.r#impl().clone(), self.context.clone())
            .await?;

        Ok(())
    }
}
