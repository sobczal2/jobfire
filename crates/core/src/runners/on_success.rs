use chrono::Utc;
use thiserror::Error;

use crate::{
    domain::job::{
        self, Job,
        context::{JobContext, JobContextData},
        pending::PendingJob,
        report::Report,
        running::RunningJob,
        successful::SuccessfulJob,
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
    #[error("on_success callback failed: {0}")]
    CallbackFailed(#[from] job::error::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub struct OnSuccessRunnerInput {
    job: Job,
    pending_job: PendingJob,
    running_job: RunningJob,
    report: Report,
}

impl OnSuccessRunnerInput {
    pub fn new(job: Job, pending_job: PendingJob, running_job: RunningJob, report: Report) -> Self {
        Self {
            job,
            pending_job,
            running_job,
            report,
        }
    }
}

pub struct OnSuccessRunner<TData: JobContextData> {
    storage: Storage,
    context: JobContext<TData>,
    job_actions_registry: JobActionsRegistry<TData>,
}

impl<TData: JobContextData> Clone for OnSuccessRunner<TData> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            context: self.context.clone(),
            job_actions_registry: self.job_actions_registry.clone(),
        }
    }
}

impl<TData: JobContextData> OnSuccessRunner<TData> {
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

    pub async fn run(&self, input: &OnSuccessRunnerInput) {
        if let Err(error) = self.run_internal(input).await {
            log::error!("error during on_success callback run: {error}");
        }
    }

    async fn run_internal(&self, input: &OnSuccessRunnerInput) -> Result<()> {
        let successful_job = SuccessfulJob::new(
            *input.job.id(),
            *input.pending_job.scheduled_at(),
            Utc::now(),
            input.report.clone(),
        );

        self.storage
            .successful_job_repo()
            .add(successful_job)
            .await?;

        let job_actions = self
            .job_actions_registry
            .get(input.job.r#impl().name())
            .ok_or(Error::JobActionsNotFound)?;

        job_actions
            .on_success(input.job.r#impl().clone(), self.context.clone())
            .await?;

        Ok(())
    }
}
