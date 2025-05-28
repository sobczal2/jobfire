use std::{pin::Pin, sync::Arc};

use crate::domain::job::{
    context::{Context, ContextData},
    error::{JobError, JobResult},
    r#impl::{JobImpl, SerializedJobImpl},
    report::Report,
};

pub type RunFn<TData> = Arc<
    dyn Fn(
            SerializedJobImpl,
            Context<TData>,
        ) -> Pin<Box<dyn Future<Output = JobResult<Report>> + Send>>
        + Send
        + Sync,
>;
pub type OnSuccessFn<TData> = Arc<
    dyn Fn(SerializedJobImpl, Context<TData>) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;
pub type OnFailFn<TData> = Arc<
    dyn Fn(SerializedJobImpl, Context<TData>) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

pub struct JobActions<TData: ContextData> {
    run_fn: RunFn<TData>,
    on_success_fn: OnSuccessFn<TData>,
    on_fail_fn: OnFailFn<TData>,
}

impl<TData: ContextData> Clone for JobActions<TData> {
    fn clone(&self) -> Self {
        Self {
            run_fn: self.run_fn.clone(),
            on_success_fn: self.on_success_fn.clone(),
            on_fail_fn: self.on_fail_fn.clone(),
        }
    }
}

impl<TData: ContextData> JobActions<TData> {
    pub fn new(
        run: RunFn<TData>,
        on_success: OnSuccessFn<TData>,
        on_fail: OnFailFn<TData>,
    ) -> Self {
        Self {
            run_fn: run,
            on_success_fn: on_success,
            on_fail_fn: on_fail,
        }
    }
    pub fn from_job_impl<TJobImpl: JobImpl<TData>>() -> Self {
        let run: RunFn<TData> = Arc::new(
            |serialized_job_impl: SerializedJobImpl, job_context: Context<TData>| {
                Box::pin(async move {
                    let job_impl = serialized_job_impl
                        .deserialize::<TData, TJobImpl>()
                        .map_err(|_| JobError::JobImplBuildFailed);
                    match job_impl {
                        Ok(job_impl) => job_impl.run(job_context).await,
                        Err(e) => {
                            log::error!("failed to run job action");
                            Err(e)
                        }
                    }
                })
            },
        );

        let on_success: OnSuccessFn<TData> = Arc::new(
            |serialized_job_impl: SerializedJobImpl, job_context: Context<TData>| {
                Box::pin(async move {
                    let job_impl = serialized_job_impl
                        .deserialize::<TData, TJobImpl>()
                        .map_err(|_| JobError::JobImplBuildFailed);
                    match job_impl {
                        Ok(job_impl) => job_impl.on_success(job_context).await,
                        Err(e) => {
                            log::error!("failed to run on_success job action: {:?}", e);
                        }
                    }
                })
            },
        );

        let on_fail: OnFailFn<TData> = Arc::new(
            |serialized_job_impl: SerializedJobImpl, job_context: Context<TData>| {
                Box::pin(async move {
                    let job_impl = serialized_job_impl
                        .deserialize::<TData, TJobImpl>()
                        .map_err(|_| JobError::JobImplBuildFailed);
                    match job_impl {
                        Ok(job_impl) => job_impl.on_fail(job_context).await,
                        Err(e) => {
                            log::error!("failed to run on_fail job action: {:?}", e);
                        }
                    }
                })
            },
        );

        Self::new(run, on_success, on_fail)
    }

    pub fn get_run_fn(&self) -> RunFn<TData> {
        self.run_fn.clone()
    }

    pub fn get_on_success_fn(&self) -> OnSuccessFn<TData> {
        self.on_success_fn.clone()
    }

    pub fn get_on_fail_fn(&self) -> OnFailFn<TData> {
        self.on_fail_fn.clone()
    }
}
