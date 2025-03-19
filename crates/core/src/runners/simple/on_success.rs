use async_trait::async_trait;
use thiserror::Error;

use crate::{
    domain::job::{
        self,
        context::{JobContext, JobContextData},
    },
    registries::job_actions::JobActionsRegistry,
    runners::on_success::{OnSuccessRunner, OnSuccessRunnerInput},
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

pub struct SimpleOnSuccessRunner<TData: JobContextData> {
    storage: Storage,
    context: JobContext<TData>,
    job_actions_registry: JobActionsRegistry<TData>,
}

#[async_trait]
impl<TData: JobContextData> OnSuccessRunner<TData> for SimpleOnSuccessRunner<TData> {
    async fn run(&self, input: &OnSuccessRunnerInput) {
        if let Err(error) = self.run_internal(input).await {
            log::error!("error during on_success callback run: {error}");
        }
    }
}

impl<TData: JobContextData> SimpleOnSuccessRunner<TData> {
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
    async fn run_internal(&self, input: &OnSuccessRunnerInput) -> Result<()> {
        let job_actions = self
            .job_actions_registry
            .get(input.pending_job().impl_name())
            .ok_or(Error::JobActionsNotFound)?;

        job_actions
            .on_success(input.pending_job(), self.context.clone())
            .await?;

        // TODO: save success job

        Ok(())
    }
}
