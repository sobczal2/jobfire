use chrono::Utc;
use thiserror::Error;

use crate::{
    domain::job::{
        Job,
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
    #[error("corresponding job not found")]
    CorrespondingJobNotFound,
    #[error("job actions not found")]
    JobActionsNotFound,
}

type Result<T> = std::result::Result<T, Error>;

pub struct JobRunner<TData: JobContextData> {
    storage: Storage,
    context: JobContext<TData>,
    job_actions_registry: JobActionsRegistry<TData>,
    on_success_runner: OnSuccessRunner<TData>,
    on_fail_runner: OnFailRunner<TData>,
}

impl<TData: JobContextData> Clone for JobRunner<TData> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            context: self.context.clone(),
            job_actions_registry: self.job_actions_registry.clone(),
            on_success_runner: self.on_success_runner.clone(),
            on_fail_runner: self.on_fail_runner.clone(),
        }
    }
}

impl<TData: JobContextData> JobRunner<TData> {
    pub fn new(
        storage: Storage,
        context: JobContext<TData>,
        job_actions_registry: JobActionsRegistry<TData>,
    ) -> Self {
        Self {
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
        }
    }

    pub async fn run(&self, pending_job: PendingJob) {
        if let Err(error) = self.run_internal(pending_job).await {
            log::error!("error during job run: {error}");
        }
    }

    async fn run_internal(&self, pending_job: PendingJob) -> Result<()> {
        let job = self.get_job(pending_job.job_id()).await?;
        self.save_running_job(&job).await?;

        let job_actions = self
            .job_actions_registry
            .get(job.r#impl().name())
            .ok_or(Error::JobActionsNotFound)?;

        let run_result = job_actions
            .run(job.r#impl().clone(), self.context.clone())
            .await;

        let running_job = self.storage.running_job_repo().pop(job.id()).await?;

        match run_result {
            Ok(report) => {
                self.on_success_runner
                    .run(&OnSuccessRunnerInput::new(
                        job.clone(),
                        pending_job.clone(),
                        running_job,
                        report,
                    ))
                    .await;
            }
            Err(error) => {
                self.on_fail_runner
                    .run(&OnFailRunnerInput::new(
                        job.clone(),
                        pending_job.clone(),
                        running_job,
                        error,
                    ))
                    .await;
            }
        }
        Ok(())
    }

    async fn save_running_job(&self, job: &Job) -> Result<()> {
        let running_job = RunningJob::new(*job.id(), Utc::now());
        self.storage.running_job_repo().add(&running_job).await?;
        Ok(())
    }

    async fn get_job(&self, job_id: &JobId) -> Result<Job> {
        let job = self.storage.job_repo().get(job_id).await?;
        match job {
            Some(job) => Ok(job),
            None => Err(Error::CorrespondingJobNotFound),
        }
    }
}
