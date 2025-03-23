use std::sync::Arc;

use chrono::Utc;
use ephemeral_fn_registry::{EphemeralFnRegistry, EphemeralRunFn};
use r#impl::{EphemeralJobId, EphemeralJobImpl};
use jobfire_core::{
    async_trait,
    builders::job_manager::JobManagerBuilder,
    domain::job::{
        context::{Context, ContextData},
        error::JobResult,
        id::JobId,
        report::Report,
    },
    managers::{self, job_manager::JobManager},
};

pub mod ephemeral_fn_registry;
pub mod r#impl;

pub trait AddEphemeralExtension {
    fn add_ephemeral_extension(&self) -> Self;
}

impl<TData: ContextData> AddEphemeralExtension for JobManagerBuilder<TData> {
    fn add_ephemeral_extension(&self) -> Self {
        self.register_job_impl::<EphemeralJobImpl>();
        self.add_service(EphemeralFnRegistry::<TData>::new(Default::default()));
        self.clone()
    }
}

#[async_trait]
pub trait ScheduleEphemeralJob<TData: ContextData> {
    async fn schedule_ephemeral_job<F, Fut>(
        &self,
        run_closure: F,
    ) -> managers::job_manager::Result<JobId>
    where
        F: Fn(Context<TData>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = JobResult<Report>> + Send + 'static;
}

#[async_trait]
impl<TData: ContextData> ScheduleEphemeralJob<TData> for JobManager<TData> {
    async fn schedule_ephemeral_job<F, Fut>(
        &self,
        run_closure: F,
    ) -> managers::job_manager::Result<JobId>
    where
        F: Fn(Context<TData>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = JobResult<Report>> + Send + 'static,
    {
        let ephemeral_job_id = EphemeralJobId::new();
        let run_fn: EphemeralRunFn<TData> = Arc::new(move |ctx| Box::pin(run_closure(ctx)));
        self.get_service::<EphemeralFnRegistry<TData>>()
            .unwrap()
            .add(&ephemeral_job_id, run_fn.into())
            .await
            .unwrap();
        self.schedule(EphemeralJobImpl::new(ephemeral_job_id), Utc::now())
            .await
    }
}
