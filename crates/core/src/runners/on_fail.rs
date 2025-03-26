use chrono::Utc;
use thiserror::Error;

use crate::{
    domain::{
        job::{
            Job,
            context::{Context, ContextData},
            error::JobError,
            pending::PendingJob,
            running::RunningJob,
        },
        run::failed::FailedRun,
    },
    registries::job_actions::JobActionsRegistry,
    services::verify::{ServiceMissing, VerifyService},
    storage::{self, Storage},
    verify_services,
};

#[derive(Error, Debug)]
enum Error {
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
    #[error("job actions not found")]
    JobActionsNotFound,
}

type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub struct OnFailRunnerInput {
    job: Job,
    pending_job: PendingJob,
    running_job: RunningJob,
    error: JobError,
}

impl OnFailRunnerInput {
    pub fn new(
        job: Job,
        pending_job: PendingJob,
        running_job: RunningJob,
        error: JobError,
    ) -> Self {
        Self {
            job,
            pending_job,
            running_job,
            error,
        }
    }
}

pub struct OnFailRunner<TData: ContextData> {
    context: Context<TData>,
}

impl<TData: ContextData> VerifyService<TData> for OnFailRunner<TData> {
    fn verify(
        &self,
        services: &crate::services::Services<TData>,
    ) -> std::result::Result<(), ServiceMissing> {
        verify_services!(services, JobActionsRegistry<TData>, Storage);
        Ok(())
    }
}

impl<TData: ContextData> Clone for OnFailRunner<TData> {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
        }
    }
}

impl<TData: ContextData> OnFailRunner<TData> {
    pub fn new(context: Context<TData>) -> Self {
        Self { context }
    }

    pub async fn run(&self, input: &OnFailRunnerInput) {
        if let Err(error) = self.run_internal(input).await {
            log::error!("error during on_fail callback run: {error}");
        }
    }

    async fn run_internal(&self, input: &OnFailRunnerInput) -> Result<()> {
        let failed_run = FailedRun::new(
            *input.running_job.run_id(),
            *input.job.id(),
            *input.pending_job.scheduled_at(),
            Utc::now(),
            input.error.clone(),
        );

        self.context
            .get_required_service::<Storage>()
            .failed_run_repo()
            .add(failed_run)
            .await?;

        let job_actions = self
            .context
            .get_required_service::<JobActionsRegistry<TData>>()
            .get(input.job.r#impl().name())
            .ok_or(Error::JobActionsNotFound)?;

        job_actions
            .on_fail(input.job.r#impl().clone(), self.context.clone())
            .await;

        Ok(())
    }
}
