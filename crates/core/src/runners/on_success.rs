use crate::{
    domain::{
        job::{
            self, Job,
            context::{Context, ContextData},
            pending::PendingJob,
            report::Report,
            running::RunningJob,
        },
        run::successful::SuccessfulRun,
    },
    registries::job_actions::JobActionsRegistry,
    services::{
        time::{AnyClock, Clock},
        verify::{ServiceMissing, VerifyService},
    },
    storage::{self, Storage},
    verify_services,
};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
    #[error("job actions not found")]
    JobActionsNotFound,
    #[error("on_success callback failed: {0}")]
    CallbackFailed(#[from] job::error::JobError),
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

pub struct OnSuccessRunner<TData: ContextData> {
    context: Context<TData>,
}

impl<TData: ContextData> VerifyService for OnSuccessRunner<TData> {
    fn verify(
        &self,
        services: &crate::services::Services,
    ) -> std::result::Result<(), ServiceMissing> {
        verify_services!(services, JobActionsRegistry<TData>, Storage, AnyClock);
        Ok(())
    }
}

impl<TData: ContextData> Clone for OnSuccessRunner<TData> {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
        }
    }
}

impl<TData: ContextData> OnSuccessRunner<TData> {
    pub fn new(context: Context<TData>) -> Self {
        Self { context }
    }

    pub async fn run(&self, input: &OnSuccessRunnerInput) {
        if let Err(error) = self.run_internal(input).await {
            log::error!("error during on_success callback run: {error}");
        }
    }

    async fn run_internal(&self, input: &OnSuccessRunnerInput) -> Result<()> {
        let now = self.context.get_required_service::<AnyClock>().utc_now();
        let successful_run = SuccessfulRun::new(
            input.running_job.run_id(),
            input.job.id(),
            input.pending_job.scheduled_at(),
            now,
            input.report.clone(),
        );

        self.context
            .get_required_service::<Storage>()
            .successful_run_repo()
            .add(successful_run)
            .await?;

        let job_actions = self
            .context
            .get_required_service::<JobActionsRegistry<TData>>()
            .get(input.job.r#impl().name())
            .ok_or(Error::JobActionsNotFound)?;

        (job_actions.get_on_success_fn())(input.job.r#impl().clone(), self.context.clone()).await;

        Ok(())
    }
}
