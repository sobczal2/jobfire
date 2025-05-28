use chrono::Utc;
use thiserror::Error;

use crate::{
    domain::{
        job::{
            Job,
            context::{Context, ContextData},
            error::{JobError, JobResult},
            id::JobId,
            pending::PendingJob,
            report::Report,
            running::RunningJob,
        },
        run::{
            id::RunId,
            job_actions::{JobActions, RunFn},
        },
    },
    registries::{job_actions::JobActionsRegistry, policies::PolicyRegistry},
    services::verify::{ServiceMissing, VerifyService},
    storage::{self, Storage},
    verify_services,
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

pub struct JobRunner<TData: ContextData> {
    context: Context<TData>,
}

impl<TData: ContextData> VerifyService for JobRunner<TData> {
    fn verify(
        &self,
        services: &crate::services::Services,
    ) -> std::result::Result<(), ServiceMissing> {
        verify_services!(
            services,
            JobActionsRegistry<TData>,
            Storage,
            OnSuccessRunner<TData>,
            OnFailRunner<TData>
        );
        Ok(())
    }
}

impl<TData: ContextData> Clone for JobRunner<TData> {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
        }
    }
}

impl<TData: ContextData> JobRunner<TData> {
    pub fn new(context: Context<TData>) -> Self {
        Self {
            context: context.clone(),
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
            .context
            .get_required_service::<JobActionsRegistry<TData>>()
            .get(job.r#impl().name())
            .ok_or(Error::JobActionsNotFound)?;

        let policy_registry = self.context.get_required_service::<PolicyRegistry<TData>>();

        let run_result = self
            .run_job_with_policies(job_actions, policy_registry, &job)
            .await;

        let running_job = self
            .context
            .get_required_service::<Storage>()
            .running_job_repo()
            .delete(job.id())
            .await?;

        self.context
            .get_required_service::<Storage>()
            .job_repo()
            .update_policies(job.id(), job.policies().clone())
            .await?;

        match run_result {
            Ok(report) => {
                self.context
                    .get_required_service::<OnSuccessRunner<TData>>()
                    .run(&OnSuccessRunnerInput::new(
                        job.clone(),
                        pending_job.clone(),
                        running_job,
                        report,
                    ))
                    .await;
            }
            Err(error) => {
                self.context
                    .get_required_service::<OnFailRunner<TData>>()
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

    async fn run_job_with_policies(
        &self,
        job_actions: JobActions<TData>,
        policy_registry: PolicyRegistry<TData>,
        job: &Job,
    ) -> JobResult<Report> {
        let mut run_fn: RunFn<TData> = job_actions.get_run_fn();

        for policy_names in job.policies().names() {
            run_fn = policy_registry
                .wrap_run(policy_names.clone(), run_fn, job.policies().data().clone())
                .map_err(|_| JobError::PolicyNotFound)?;
        }

        run_fn(job.r#impl().clone(), self.context.clone()).await
    }

    async fn save_running_job(&self, job: &Job) -> Result<()> {
        let running_job = RunningJob::new(*job.id(), RunId::default(), Utc::now());
        self.context
            .get_required_service::<Storage>()
            .running_job_repo()
            .add(running_job)
            .await?;

        Ok(())
    }

    async fn get_job(&self, job_id: &JobId) -> Result<Job> {
        let job = self
            .context
            .get_required_service::<Storage>()
            .job_repo()
            .get(job_id)
            .await?;

        match job {
            Some(job) => Ok(job),
            None => Err(Error::CorrespondingJobNotFound),
        }
    }
}
