#![allow(type_alias_bounds)]

use chrono::{DateTime, Utc};
use jobfire_core::{
    async_trait,
    domain::job::{
        context::ContextData,
        r#impl::{JobImpl, SerializedJobImpl},
    },
    registries::job_actions::{OnFailFn, OnSuccessFn, RunFn},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type SerializedEphemeralJobImpl = SerializedJobImpl;

#[derive(Serialize, Deserialize, Debug, Hash)]
pub struct EphemeralJobId(Uuid);

#[derive(Serialize, Deserialize, Debug)]
pub struct EphemeralJob {
    ephemeral_job_id: EphemeralJobId,
    name: String,
    scheduled_at: DateTime<Utc>,
    serialized_impl: SerializedEphemeralJobImpl,
}

impl EphemeralJob {
    pub fn new(
        ephemeral_job_id: EphemeralJobId,
        name: String,
        scheduled_at: DateTime<Utc>,
        serialized_impl: SerializedEphemeralJobImpl,
    ) -> Self {
        Self {
            ephemeral_job_id,
            name,
            scheduled_at,
            serialized_impl,
        }
    }
}

#[async_trait]
impl<TData: ContextData> JobImpl<TData> for EphemeralJob {
    fn name() -> jobfire_core::domain::job::r#impl::JobImplName {
        todo!()
    }

    async fn run(
        &self,
        context: jobfire_core::domain::job::context::Context<TData>,
    ) -> jobfire_core::domain::job::error::JobResult<jobfire_core::domain::job::report::Report>
    {
        todo!()
    }

    async fn on_fail(&self, context: jobfire_core::domain::job::context::Context<TData>) {
        todo!()
    }

    async fn on_success(&self, context: jobfire_core::domain::job::context::Context<TData>) {
        todo!()
    }
}
