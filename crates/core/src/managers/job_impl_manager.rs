use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, RwLock},
};

use crate::domain::job::{JobContext, JobError, JobImpl, JobImplName, PendingJob, Report};

pub type RunImplFn<TJobContext: JobContext> = Arc<
    dyn Fn(
            PendingJob,
            TJobContext,
        ) -> Pin<Box<dyn Future<Output = Result<Report, JobError>> + Send>>
        + Send
        + Sync,
>;
pub type OnSuccessFn<TJobContext: JobContext> =
    Arc<dyn Fn(PendingJob, TJobContext) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;
pub type OnFailImplFn<TJobContext: JobContext> =
    Arc<dyn Fn(PendingJob, TJobContext) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Clone)]
struct JobActions<TJobContext: JobContext> {
    run: RunImplFn<TJobContext>,
    on_success: OnSuccessFn<TJobContext>,
    on_fail: OnFailImplFn<TJobContext>,
}

pub struct JobImplManager<TJobContext: JobContext> {
    job_actions_map: HashMap<JobImplName, JobActions<TJobContext>>,
}

impl<TJobContext: JobContext> Clone for JobImplManager<TJobContext> {
    fn clone(&self) -> Self {
        Self {
            job_actions_map: self.job_actions_map.clone(),
        }
    }
}

impl<TJobContext: JobContext> JobImplManager<TJobContext> {
    pub fn new(job_actions_map: HashMap<JobImplName, JobActions<TJobContext>>) -> Self {
        Self {
            job_actions_map: job_actions_map,
        }
    }

    pub fn register<TJobImpl: JobImpl<TJobContext>>(&mut self) {
        let run: RunImplFn<TJobContext> = Arc::new(|pending_job: PendingJob, job_context| {
            Box::pin(async move {
                let job_impl = pending_job
                    .build_impl::<TJobContext, TJobImpl>()
                    .map_err(|e| JobError::new(e.to_string()));
                match job_impl {
                    Ok(job_impl) => job_impl.run(job_context).await,
                    Err(e) => Err(e),
                }
            })
        });

        let on_success: OnSuccessFn<TJobContext> =
            Arc::new(|pending_job: PendingJob, job_context| {
                Box::pin(async move {
                    let job_impl = pending_job
                        .build_impl::<TJobContext, TJobImpl>()
                        .map_err(|e| JobError::new(e.to_string()));
                    match job_impl {
                        Ok(job_impl) => job_impl.on_success(job_context).await,
                        Err(e) => {
                            log::error!("failed to run on_fail job action");
                        }
                    }
                })
            });

        let on_fail: OnFailImplFn<TJobContext> =
            Arc::new(|pending_job: PendingJob, job_context| {
                Box::pin(async move {
                    let job_impl = pending_job
                        .build_impl::<TJobContext, TJobImpl>()
                        .map_err(|e| JobError::new(e.to_string()));
                    match job_impl {
                        Ok(job_impl) => job_impl.on_fail(job_context).await,
                        Err(e) => {
                            log::error!("failed to run on_fail job action");
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

    pub async fn run(
        &self,
        pending_job: PendingJob,
        job_context: TJobContext,
    ) -> Result<Report, JobError> {
        match self.job_actions_map.get(pending_job.impl_name()) {
            Some(job_actions) => (job_actions.run)(pending_job, job_context).await,
            None => panic!(), // TODO: handle
        }
    }

    pub async fn on_success(&self, pending_job: PendingJob, job_context: TJobContext) {
        match self.job_actions_map.get(pending_job.impl_name()) {
            Some(job_actions) => (job_actions.on_success)(pending_job, job_context).await,
            None => panic!(), // TODO: handle
        }
    }

    pub async fn on_fail(&self, pending_job: PendingJob, job_context: TJobContext) {
        match self.job_actions_map.get(pending_job.impl_name()) {
            Some(job_actions) => (job_actions.on_fail)(pending_job, job_context).await,
            None => panic!(), // TODO: handle
        }
    }
}

impl<TJobContext: JobContext> Default for JobImplManager<TJobContext> {
    fn default() -> Self {
        Self {
            job_actions_map: Default::default(),
        }
    }
}
