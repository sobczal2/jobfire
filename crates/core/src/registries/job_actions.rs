#![allow(type_alias_bounds)]

use std::{collections::HashMap, pin::Pin, sync::Arc};

use thiserror::Error;

use crate::domain::job::{
    self,
    context::{JobContext, JobContextData},
    r#impl::{JobImplName, SerializedJobImpl},
    report::Report,
};

pub type RunFn<TData: JobContextData> = Arc<
    dyn Fn(
            SerializedJobImpl,
            JobContext<TData>,
        ) -> Pin<Box<dyn Future<Output = job::error::Result<Report>> + Send>>
        + Send
        + Sync,
>;
pub type OnSuccessFn<TData: JobContextData> = Arc<
    dyn Fn(
            SerializedJobImpl,
            JobContext<TData>,
        ) -> Pin<Box<dyn Future<Output = job::error::Result<()>> + Send>>
        + Send
        + Sync,
>;
pub type OnFailFn<TData: JobContextData> = Arc<
    dyn Fn(
            SerializedJobImpl,
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

pub struct JobActions<TData: JobContextData> {
    run: RunFn<TData>,
    on_success: OnSuccessFn<TData>,
    on_fail: OnFailFn<TData>,
}

impl<TData: JobContextData> JobActions<TData> {
    pub fn new(
        run: RunFn<TData>,
        on_success: OnSuccessFn<TData>,
        on_fail: OnFailFn<TData>,
    ) -> Self {
        Self {
            run,
            on_success,
            on_fail,
        }
    }
}

impl<TData: JobContextData> Clone for JobActions<TData> {
    fn clone(&self) -> Self {
        Self {
            run: self.run.clone(),
            on_success: self.on_success.clone(),
            on_fail: self.on_fail.clone(),
        }
    }
}

impl<TData: JobContextData> JobActions<TData> {
    pub async fn run(
        &self,
        serialized_job_impl: SerializedJobImpl,
        job_context: JobContext<TData>,
    ) -> job::error::Result<Report> {
        (self.run.clone())(serialized_job_impl, job_context).await
    }

    pub async fn on_success(
        &self,
        serialized_job_impl: SerializedJobImpl,
        job_context: JobContext<TData>,
    ) -> job::error::Result<()> {
        (self.on_success.clone())(serialized_job_impl, job_context).await
    }

    pub async fn on_fail(
        &self,
        serialized_job_impl: SerializedJobImpl,
        job_context: JobContext<TData>,
    ) -> job::error::Result<()> {
        (self.on_fail.clone())(serialized_job_impl, job_context).await
    }
}

pub struct JobActionsRegistry<TData: JobContextData> {
    inner: Arc<JobActionsRegistryInner<TData>>,
}

impl<TData: JobContextData> Clone for JobActionsRegistry<TData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct JobActionsRegistryInner<TData: JobContextData> {
    job_actions_map: HashMap<JobImplName, JobActions<TData>>,
}

impl<TData: JobContextData> JobActionsRegistry<TData> {
    pub fn new(job_actions_map: HashMap<JobImplName, JobActions<TData>>) -> Self {
        Self {
            inner: Arc::new(JobActionsRegistryInner { job_actions_map }),
        }
    }

    pub fn get(&self, job_impl_name: &JobImplName) -> Option<JobActions<TData>> {
        self.inner.job_actions_map.get(job_impl_name).cloned()
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
