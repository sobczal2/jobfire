use std::sync::Arc;

use chrono::{DateTime, Utc};
use ephemeral_fn_registry::{
    EphemeralActions, EphemeralFnRegistry, EphemeralOnFailFn, EphemeralOnSuccessFn, EphemeralRunFn,
};
use r#impl::{EphemeralJobId, EphemeralJobImpl};
use jobfire_core::{
    async_trait,
    domain::job::{
        Job,
        context::{Context, ContextData},
        error::JobResult,
        id::JobId,
        report::Report,
    },
    managers::{self, job_manager::JobManager},
    services::Services,
};

pub mod ephemeral_fn_registry;
pub mod r#impl;

pub trait AddEphemeralExtension {
    fn add_ephemeral_extension<TData: ContextData>(&self) -> Self;
}

impl AddEphemeralExtension for Services {
    fn add_ephemeral_extension<TData: ContextData>(&self) -> Self {
        self.add_service(EphemeralFnRegistry::<TData>::default())
            .clone()
    }
}

pub trait RegisterEphemeralJob {
    fn register_ephemeral_job(&mut self) -> Self;
}

#[async_trait]
pub trait ScheduleEphemeralJob<TData: ContextData> {
    async fn schedule_ephemeral_job<FRun, FutRun, FOnSuccess, FutOnSuccess, FOnFail, FutOnFail>(
        &self,
        run_closure: FRun,
        on_success_closure: FOnSuccess,
        on_fail_closure: FOnFail,
        at: DateTime<Utc>,
    ) -> managers::job_manager::Result<JobId>
    where
        FRun: Fn(Context<TData>) -> FutRun + Clone + Send + Sync + 'static,
        FutRun: Future<Output = JobResult<Report>> + Send + 'static,
        FOnSuccess: Fn(Context<TData>) -> FutOnSuccess + Clone + Send + Sync + 'static,
        FutOnSuccess: Future<Output = ()> + Send + 'static,
        FOnFail: Fn(Context<TData>) -> FutOnFail + Clone + Send + Sync + 'static,
        FutOnFail: Future<Output = ()> + Send + 'static;

    async fn schedule_ephemeral_job_now<
        FRun,
        FutRun,
        FOnSuccess,
        FutOnSuccess,
        FOnFail,
        FutOnFail,
    >(
        &self,
        run_closure: FRun,
        on_success_closure: FOnSuccess,
        on_fail_closure: FOnFail,
    ) -> managers::job_manager::Result<JobId>
    where
        FRun: Fn(Context<TData>) -> FutRun + Clone + Send + Sync + 'static,
        FutRun: Future<Output = JobResult<Report>> + Send + 'static,
        FOnSuccess: Fn(Context<TData>) -> FutOnSuccess + Clone + Send + Sync + 'static,
        FutOnSuccess: Future<Output = ()> + Send + 'static,
        FOnFail: Fn(Context<TData>) -> FutOnFail + Clone + Send + Sync + 'static,
        FutOnFail: Future<Output = ()> + Send + 'static,
    {
        self.schedule_ephemeral_job(run_closure, on_success_closure, on_fail_closure, Utc::now())
            .await
    }

    async fn schedule_simple_ephemeral_job<F, Fut>(
        &self,
        run_closure: F,
        at: DateTime<Utc>,
    ) -> managers::job_manager::Result<JobId>
    where
        F: Fn(Context<TData>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = JobResult<Report>> + Send + 'static,
    {
        self.schedule_ephemeral_job(run_closure, async |_| {}, async |_| {}, at)
            .await
    }

    async fn schedule_simple_ephemeral_job_now<F, Fut>(
        &self,
        run_closure: F,
    ) -> managers::job_manager::Result<JobId>
    where
        F: Fn(Context<TData>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = JobResult<Report>> + Send + 'static,
    {
        self.schedule_simple_ephemeral_job(run_closure, Utc::now())
            .await
    }
}

#[async_trait]
impl<TData: ContextData> ScheduleEphemeralJob<TData> for JobManager<TData> {
    async fn schedule_ephemeral_job<FRun, FutRun, FOnSuccess, FutOnSuccess, FOnFail, FutOnFail>(
        &self,
        run_closure: FRun,
        on_success_closure: FOnSuccess,
        on_fail_closure: FOnFail,
        at: DateTime<Utc>,
    ) -> managers::job_manager::Result<JobId>
    where
        FRun: Fn(Context<TData>) -> FutRun + Clone + Send + Sync + 'static,
        FutRun: Future<Output = JobResult<Report>> + Send + 'static,
        FOnSuccess: Fn(Context<TData>) -> FutOnSuccess + Clone + Send + Sync + 'static,
        FutOnSuccess: Future<Output = ()> + Send + 'static,
        FOnFail: Fn(Context<TData>) -> FutOnFail + Clone + Send + Sync + 'static,
        FutOnFail: Future<Output = ()> + Send + 'static,
    {
        let ephemeral_fn_registry = self
            .context()
            .get_service::<EphemeralFnRegistry<TData>>()
            .ok_or(managers::job_manager::Error::ServiceMissing(
                "EphemeralFnRegistry".to_owned(),
            ))?;
        let ephemeral_job_id = EphemeralJobId::new();
        let ephemeral_job_impl = EphemeralJobImpl::new(ephemeral_job_id);

        let ephemeral_run_fn: EphemeralRunFn<TData> = Arc::new(move |context| {
            let closure = run_closure.clone();
            Box::pin(async move { closure(context).await })
        });

        let ephemeral_on_success_closure: EphemeralOnSuccessFn<TData> = Arc::new(move |context| {
            let closure = on_success_closure.clone();
            Box::pin(async move { closure(context).await })
        });

        let ephemeral_on_fail_closure: EphemeralOnFailFn<TData> = Arc::new(move |context| {
            let closure = on_fail_closure.clone();
            Box::pin(async move { closure(context).await })
        });

        let ephemeral_actions = EphemeralActions::new(
            ephemeral_run_fn,
            ephemeral_on_success_closure,
            ephemeral_on_fail_closure,
        );

        ephemeral_fn_registry
            .add(&ephemeral_job_id, ephemeral_actions)
            .await
            .map_err(|e| managers::job_manager::Error::InternalError(e.to_string()))?;

        self.schedule(
            Job::from_impl::<TData>(ephemeral_job_impl, Vec::new())
                .map_err(|e| managers::job_manager::Error::InternalError(e.to_string()))?,
            at,
        )
        .await
    }
}
