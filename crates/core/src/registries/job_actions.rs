use std::{collections::HashMap, pin::Pin, sync::Arc};

use thiserror::Error;

use crate::domain::job::{
    self,
    context::{JobContext, JobContextData},
    r#impl::{JobImpl, JobImplName},
    pending::PendingJob,
    report::Report,
};

pub type RunFn<TData: JobContextData> = Arc<
    dyn Fn(
            PendingJob,
            JobContext<TData>,
        ) -> Pin<Box<dyn Future<Output = job::error::Result<Report>> + Send>>
        + Send
        + Sync,
>;
pub type OnSuccessFn<TData: JobContextData> = Arc<
    dyn Fn(
            PendingJob,
            JobContext<TData>,
        ) -> Pin<Box<dyn Future<Output = job::error::Result<()>> + Send>>
        + Send
        + Sync,
>;
pub type OnFailFn<TData: JobContextData> = Arc<
    dyn Fn(
            PendingJob,
            JobContext<TData>,
        ) -> Pin<Box<dyn Future<Output = job::error::Result<()>> + Send>>
        + Send
        + Sync,
>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to retrieve fn")]
    FnNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct JobActions<TData: JobContextData> {
    run: RunFn<TData>,
    on_success: OnSuccessFn<TData>,
    on_fail: OnFailFn<TData>,
}

impl<TData: JobContextData> JobActions<TData> {
    pub async fn run(
        &self,
        pending_job: &PendingJob,
        job_context: JobContext<TData>,
    ) -> job::error::Result<Report> {
        (self.run.clone())(pending_job.clone(), job_context).await
    }

    pub async fn on_success(
        &self,
        pending_job: &PendingJob,
        job_context: JobContext<TData>,
    ) -> job::error::Result<()> {
        (self.on_success.clone())(pending_job.clone(), job_context).await
    }

    pub async fn on_fail(
        &self,
        pending_job: &PendingJob,
        job_context: JobContext<TData>,
    ) -> job::error::Result<()> {
        (self.on_fail.clone())(pending_job.clone(), job_context).await
    }
}

#[derive(Clone)]
pub struct JobActionsRegistry<TData: JobContextData> {
    job_actions_map: HashMap<JobImplName, JobActions<TData>>,
}

impl<TData: JobContextData> JobActionsRegistry<TData> {
    pub fn new(job_actions_map: HashMap<JobImplName, JobActions<TData>>) -> Self {
        Self { job_actions_map }
    }

    pub fn register<TJobImpl: JobImpl<TData>>(&mut self) {
        let run: RunFn<TData> =
            Arc::new(|pending_job: PendingJob, job_context: JobContext<TData>| {
                Box::pin(async move {
                    let job_impl = serde_json::from_value::<TJobImpl>(pending_job.r#impl().clone())
                        .map_err(|_| job::error::Error::JobImplBuildFailed);
                    match job_impl {
                        Ok(job_impl) => job_impl.run(job_context).await,
                        Err(e) => {
                            log::error!("failed to run job action");
                            Err(e)
                        }
                    }
                })
            });

        let on_success: OnSuccessFn<TData> =
            Arc::new(|pending_job: PendingJob, job_context: JobContext<TData>| {
                Box::pin(async move {
                    let job_impl = serde_json::from_value::<TJobImpl>(pending_job.r#impl().clone())
                        .map_err(|_| job::error::Error::JobImplBuildFailed);
                    match job_impl {
                        Ok(job_impl) => job_impl.on_success(job_context).await,
                        Err(e) => {
                            log::error!("failed to run on_success job action");
                            Err(e)
                        }
                    }
                })
            });

        let on_fail: OnFailFn<TData> =
            Arc::new(|pending_job: PendingJob, job_context: JobContext<TData>| {
                Box::pin(async move {
                    let job_impl = serde_json::from_value::<TJobImpl>(pending_job.r#impl().clone())
                        .map_err(|_| job::error::Error::JobImplBuildFailed);
                    match job_impl {
                        Ok(job_impl) => job_impl.on_fail(job_context).await,
                        Err(e) => {
                            log::error!("failed to run on_fail job action");
                            Err(e)
                        }
                    }
                })
            });

        self.job_actions_map.insert(
            TJobImpl::name(),
            JobActions {
                run,
                on_success,
                on_fail,
            },
        );
    }

    pub fn get(&self, job_impl_name: &JobImplName) -> Option<JobActions<TData>> {
        self.job_actions_map.get(&job_impl_name).cloned()
    }

    pub fn get_run_fn(&self, job_impl_name: &JobImplName) -> Option<RunFn<TData>> {
        self.get(job_impl_name).map(|ja| ja.run.clone())
    }

    pub fn get_on_success_fn(&self, job_impl_name: &JobImplName) -> Option<OnSuccessFn<TData>> {
        self.get(job_impl_name).map(|ja| ja.on_success.clone())
    }

    pub fn get_on_fail_fn(&self, job_impl_name: &JobImplName) -> Option<OnFailFn<TData>> {
        self.get(job_impl_name).map(|ja| ja.on_fail.clone())
    }
}

impl<TData: JobContextData> Default for JobActionsRegistry<TData> {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}
