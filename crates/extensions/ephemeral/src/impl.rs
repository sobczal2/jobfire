#![allow(type_alias_bounds)]

use jobfire_core::{
    async_trait,
    domain::job::{
        context::{Context, ContextData},
        error::{JobError, JobResult},
        r#impl::{JobImpl, JobImplName},
        report::Report,
    },
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ephemeral_fn_registry::EphemeralFnRegistry;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct EphemeralJobId(Uuid);

impl EphemeralJobId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EphemeralJobImpl {
    ephemeral_job_id: EphemeralJobId,
}

impl EphemeralJobImpl {
    pub fn new(ephemeral_job_id: EphemeralJobId) -> Self {
        Self { ephemeral_job_id }
    }
}

#[async_trait]
impl<TData: ContextData> JobImpl<TData> for EphemeralJobImpl {
    fn name() -> JobImplName {
        JobImplName::new("ephemeral-job")
    }

    async fn run(&self, context: Context<TData>) -> JobResult<Report> {
        let run_fn = context
            .services()
            .get_service::<EphemeralFnRegistry<TData>>()
            .unwrap()
            .get_run_fn(&self.ephemeral_job_id)
            .await;

        match run_fn {
            Some(run_fn) => (run_fn)(context).await,
            None => Err(JobError::JobImplBuildFailed),
        }
    }

    async fn on_success(&self, context: Context<TData>) {
        let on_success_fn = context
            .services()
            .get_service::<EphemeralFnRegistry<TData>>()
            .unwrap()
            .get_on_success_fn(&self.ephemeral_job_id)
            .await;

        match on_success_fn {
            Some(on_success_fn) => (on_success_fn)(context).await,
            None => log::error!("failed to found ephemeral on_success_fn"),
        }
    }

    async fn on_fail(&self, context: Context<TData>) {
        let on_fail_fn = context
            .services()
            .get_service::<EphemeralFnRegistry<TData>>()
            .unwrap()
            .get_on_success_fn(&self.ephemeral_job_id)
            .await;

        match on_fail_fn {
            Some(on_fail_fn) => (on_fail_fn)(context).await,
            None => log::error!("failed to found ephemeral on_fail_fn"),
        }
    }
}
