use thiserror::Error;

use crate::{
    domain::job::{
        self,
        context::{JobContext, JobContextData},
        pending::PendingJob,
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
    pending_job: PendingJob,
}

impl OnFailRunnerInput {
    pub fn new(pending_job: PendingJob) -> Self {
        Self { pending_job }
    }
}

pub struct OnFailRunner<TData: JobContextData> {
    storage: Storage,
    context: JobContext<TData>,
    job_actions_registry: JobActionsRegistry<TData>,
}

impl<'a, TData: JobContextData> OnFailRunner<TData> {
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
        let job_actions = self
            .job_actions_registry
            .get(input.pending_job.impl_name())
            .ok_or(Error::JobActionsNotFound)?;

        job_actions
            .on_fail(&input.pending_job, self.context.clone())
            .await?;

        // TODO: save failed job

        Ok(())
    }
}
