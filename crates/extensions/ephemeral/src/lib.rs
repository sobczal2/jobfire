use jobfire_core::{
    async_trait,
    domain::job::{
        context::{Context, ContextData},
        error::JobResult,
        id::JobId,
        report::Report,
    },
    managers::{self},
};

pub mod ephemeral_fn_registry;
pub mod r#impl;

pub trait AddEphemeralExtension {
    fn add_ephemeral_extension(&self) -> Self;
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
