use std::{collections::HashMap, pin::Pin, sync::Arc};

use thiserror::Error;

use crate::domain::job::{self, JobContext, JobImpl, JobImplName, PendingJob, Report};

pub type RunFn<TJobContext: JobContext> = Arc<
    dyn Fn(PendingJob, TJobContext) -> Pin<Box<dyn Future<Output = job::Result<Report>> + Send>>
        + Send
        + Sync,
>;
pub type OnSuccessFn<TJobContext: JobContext> = Arc<
    dyn Fn(PendingJob, TJobContext) -> Pin<Box<dyn Future<Output = job::Result<()>> + Send>>
        + Send
        + Sync,
>;
pub type OnFailFn<TJobContext: JobContext> = Arc<
    dyn Fn(PendingJob, TJobContext) -> Pin<Box<dyn Future<Output = job::Result<()>> + Send>>
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
pub struct JobActions<TJobContext: JobContext> {
    run: RunFn<TJobContext>,
    on_success: OnSuccessFn<TJobContext>,
    on_fail: OnFailFn<TJobContext>,
}

impl<TJobContext: JobContext> JobActions<TJobContext> {
    pub async fn run(
        &self,
        pending_job: &PendingJob,
        job_context: TJobContext,
    ) -> job::Result<Report> {
        (self.run.clone())(pending_job.clone(), job_context).await
    }

    pub async fn on_success(
        &self,
        pending_job: &PendingJob,
        job_context: TJobContext,
    ) -> job::Result<()> {
        (self.on_success.clone())(pending_job.clone(), job_context).await
    }

    pub async fn on_fail(
        &self,
        pending_job: &PendingJob,
        job_context: TJobContext,
    ) -> job::Result<()> {
        (self.on_fail.clone())(pending_job.clone(), job_context).await
    }
}

#[derive(Clone)]
pub struct JobActionsRegistry<TJobContext: JobContext> {
    job_actions_map: HashMap<JobImplName, JobActions<TJobContext>>,
}

impl<TJobContext: JobContext> JobActionsRegistry<TJobContext> {
    pub fn new(job_actions_map: HashMap<JobImplName, JobActions<TJobContext>>) -> Self {
        Self { job_actions_map }
    }

    pub fn register<TJobImpl: JobImpl<TJobContext>>(&mut self) {
        let run: RunFn<TJobContext> = Arc::new(|pending_job: PendingJob, job_context| {
            Box::pin(async move {
                let job_impl = serde_json::from_value::<TJobImpl>(pending_job.r#impl().clone())
                    .map_err(|_| job::Error::JobImplBuildFailed);
                match job_impl {
                    Ok(job_impl) => job_impl.run(job_context).await,
                    Err(e) => {
                        log::error!("failed to run job action");
                        Err(e)
                    }
                }
            })
        });

        let on_success: OnSuccessFn<TJobContext> =
            Arc::new(|pending_job: PendingJob, job_context| {
                Box::pin(async move {
                    let job_impl = serde_json::from_value::<TJobImpl>(pending_job.r#impl().clone())
                        .map_err(|_| job::Error::JobImplBuildFailed);
                    match job_impl {
                        Ok(job_impl) => job_impl.on_success(job_context).await,
                        Err(e) => {
                            log::error!("failed to run on_success job action");
                            Err(e)
                        }
                    }
                })
            });

        let on_fail: OnFailFn<TJobContext> = Arc::new(|pending_job: PendingJob, job_context| {
            Box::pin(async move {
                let job_impl = serde_json::from_value::<TJobImpl>(pending_job.r#impl().clone())
                    .map_err(|_| job::Error::JobImplBuildFailed);
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

    pub fn get(&self, job_impl_name: &JobImplName) -> Option<JobActions<TJobContext>> {
        self.job_actions_map.get(&job_impl_name).cloned()
    }

    pub fn get_run_fn(&self, job_impl_name: &JobImplName) -> Option<RunFn<TJobContext>> {
        self.get(job_impl_name).map(|ja| ja.run.clone())
    }

    pub fn get_on_success_fn(
        &self,
        job_impl_name: &JobImplName,
    ) -> Option<OnSuccessFn<TJobContext>> {
        self.get(job_impl_name).map(|ja| ja.on_success.clone())
    }

    pub fn get_on_fail_fn(&self, job_impl_name: &JobImplName) -> Option<OnFailFn<TJobContext>> {
        self.get(job_impl_name).map(|ja| ja.on_fail.clone())
    }
}

impl<TJobContext: JobContext> Default for JobActionsRegistry<TJobContext> {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}
