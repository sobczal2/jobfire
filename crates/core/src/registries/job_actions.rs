#![allow(type_alias_bounds)]

use std::{collections::HashMap, pin::Pin, sync::Arc};

use thiserror::Error;

use crate::{
    domain::job::{
        self,
        context::{Context, ContextData},
        r#impl::{JobImplName, SerializedJobImpl},
        report::Report,
    },
    services::verify::VerifyService,
};

pub type RunFn<TData: ContextData> = Arc<
    dyn Fn(
            SerializedJobImpl,
            Context<TData>,
        ) -> Pin<Box<dyn Future<Output = job::error::JobResult<Report>> + Send>>
        + Send
        + Sync,
>;
pub type OnSuccessFn<TData: ContextData> = Arc<
    dyn Fn(SerializedJobImpl, Context<TData>) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;
pub type OnFailFn<TData: ContextData> = Arc<
    dyn Fn(SerializedJobImpl, Context<TData>) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to retrieve fn")]
    FnNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct JobActions<TData: ContextData> {
    run: RunFn<TData>,
    on_success: OnSuccessFn<TData>,
    on_fail: OnFailFn<TData>,
}

impl<TData: ContextData> JobActions<TData> {
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

impl<TData: ContextData> Clone for JobActions<TData> {
    fn clone(&self) -> Self {
        Self {
            run: self.run.clone(),
            on_success: self.on_success.clone(),
            on_fail: self.on_fail.clone(),
        }
    }
}

impl<TData: ContextData> JobActions<TData> {
    pub async fn run(
        &self,
        r#impl: SerializedJobImpl,
        context: Context<TData>,
    ) -> job::error::JobResult<Report> {
        (self.run.clone())(r#impl, context).await
    }

    pub async fn on_success(&self, r#impl: SerializedJobImpl, context: Context<TData>) {
        (self.on_success.clone())(r#impl, context).await
    }

    pub async fn on_fail(&self, r#impl: SerializedJobImpl, context: Context<TData>) {
        (self.on_fail.clone())(r#impl, context).await
    }
}

pub struct JobActionsRegistry<TData: ContextData> {
    inner: Arc<JobActionsRegistryInner<TData>>,
}

impl<TData: ContextData> VerifyService for JobActionsRegistry<TData> {
    fn verify(
        &self,
        services: &crate::services::Services,
    ) -> std::result::Result<(), crate::services::verify::ServiceMissing> {
        Ok(())
    }
}

impl<TData: ContextData> Clone for JobActionsRegistry<TData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct JobActionsRegistryInner<TData: ContextData> {
    job_actions_map: HashMap<JobImplName, JobActions<TData>>,
}

impl<TData: ContextData> JobActionsRegistry<TData> {
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

impl<TData: ContextData> Default for JobActionsRegistry<TData> {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}
