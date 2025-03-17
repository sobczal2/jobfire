use async_trait::async_trait;
use chrono::Utc;
use thiserror::Error;

use crate::{
    domain::job::{JobContext, JobId, PendingJob, RunningJob},
    registries::job_actions::JobActionsRegistry,
    runners::{
        job::{JobRunner, JobRunnerInput},
        on_fail::{OnFailRunner, OnFailRunnerInput},
        on_success::{OnSuccessRunner, OnSuccessRunnerInput},
    },
    storage::{self, Storage},
};

#[derive(Error, Debug)]
enum Error {
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
    #[error("job actions not found")]
    JobActionsNotFound,
}

type Result<T> = std::result::Result<T, Error>;

pub struct SimpleJobRunner<TJobContext: JobContext> {
    storage: Storage,
    context: TJobContext,
    job_actions_registry: JobActionsRegistry<TJobContext>,
    on_fail_runner: Box<dyn OnFailRunner<TJobContext>>,
    on_success_runner: Box<dyn OnSuccessRunner<TJobContext>>,
}

#[async_trait]
impl<TJobContext: JobContext> JobRunner<TJobContext> for SimpleJobRunner<TJobContext> {
    async fn run(&self, job_runner_input: &JobRunnerInput) {
        if let Err(error) = self.run_internal(job_runner_input).await {
            log::error!("error during job run: {error}");
        }
    }
}

impl<TJobContext: JobContext> SimpleJobRunner<TJobContext> {
    pub fn new(
        storage: Storage,
        context: TJobContext,
        job_actions_registry: JobActionsRegistry<TJobContext>,
        on_fail_runner: Box<dyn OnFailRunner<TJobContext>>,
        on_success_runner: Box<dyn OnSuccessRunner<TJobContext>>,
    ) -> Self {
        Self {
            storage,
            context,
            job_actions_registry,
            on_fail_runner,
            on_success_runner,
        }
    }

    async fn run_internal(&self, job_runner_input: &JobRunnerInput) -> Result<()> {
        let pending_job = job_runner_input.pending_job();
        self.save_running_job(pending_job).await?;

        let job_actions = self
            .job_actions_registry
            .get(pending_job.impl_name())
            .ok_or(Error::JobActionsNotFound)?;

        let run_result = job_actions.run(pending_job, self.context.clone()).await;

        let running_job = self.remove_running_job(pending_job.id()).await?;

        match run_result {
            Ok(report) => {
                let input = OnSuccessRunnerInput::new(pending_job.clone());
                self.on_success_runner.run(&input);
            }
            Err(error) => {
                let input = OnFailRunnerInput::new(pending_job.clone());
                self.on_fail_runner.run(&input);
            }
        }
        Ok(())
    }

    async fn save_running_job(&self, pending_job: &PendingJob) -> Result<()> {
        let running_job = RunningJob::new(
            pending_job.id().clone(),
            pending_job.created_at().clone(),
            Utc::now(),
        );
        self.storage.running_job_repo().add(&running_job).await?;
        Ok(())
    }

    async fn remove_running_job(&self, job_id: &JobId) -> Result<RunningJob> {
        let running_job = self.storage.running_job_repo().pop(job_id).await?;
        Ok(running_job)
    }
}
