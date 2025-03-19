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

pub struct SimpleJobRunner<TData: JobContextData> {
    storage: Storage,
    context: JobContext<TData>,
    job_actions_registry: JobActionsRegistry<TData>,
    on_success_runner: Box<dyn OnSuccessRunner<TData>>,
    on_fail_runner: Box<dyn OnFailRunner<TData>>,
}

#[async_trait]
impl<TData: JobContextData> JobRunner<TData> for SimpleJobRunner<TData> {
    async fn run(&self, input: &JobRunnerInput) {
        if let Err(error) = self.run_internal(input).await {
            log::error!("error during job run: {error}");
        }
    }
}

impl<TData: JobContextData> SimpleJobRunner<TData> {
    pub fn new(
        storage: Storage,
        context: JobContext<TData>,
        job_actions_registry: JobActionsRegistry<TData>,
        on_success_runner: Box<dyn OnSuccessRunner<TData>>,
        on_fail_runner: Box<dyn OnFailRunner<TData>>,
    ) -> Self {
        Self {
            storage,
            context,
            job_actions_registry,
            on_success_runner,
            on_fail_runner,
        }
    }

    async fn run_internal(&self, input: &JobRunnerInput) -> Result<()> {
        let pending_job = input.pending_job();
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
                self.on_success_runner.run(&input).await;
            }
            Err(error) => {
                let input = OnFailRunnerInput::new(pending_job.clone());
                self.on_fail_runner.run(&input).await;
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
