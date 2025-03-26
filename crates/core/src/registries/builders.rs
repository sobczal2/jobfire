use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    domain::job::{
        context::{Context, ContextData},
        error::JobError,
        r#impl::{JobImpl, JobImplName, SerializedJobImpl},
    },
    services::Services,
};

use super::job_actions::{JobActions, JobActionsRegistry, OnFailFn, OnSuccessFn, RunFn};

pub struct JobActionsRegistryBuilder<TData: ContextData> {
    inner: Arc<Mutex<JobActionsRegistryBuilderInner<TData>>>,
}

impl<TData: ContextData> Clone for JobActionsRegistryBuilder<TData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TData: ContextData> Default for JobActionsRegistryBuilder<TData> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

pub struct JobActionsRegistryBuilderInner<TData: ContextData> {
    job_actions_map: HashMap<JobImplName, JobActions<TData>>,
}

impl<TData: ContextData> Default for JobActionsRegistryBuilderInner<TData> {
    fn default() -> Self {
        Self {
            job_actions_map: Default::default(),
        }
    }
}

impl<TData: ContextData> JobActionsRegistryBuilder<TData> {
    pub fn register<TJobImpl: JobImpl<TData>>(&mut self) -> Self {
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

        self.inner
            .lock()
            .unwrap()
            .job_actions_map
            .insert(TJobImpl::name(), JobActions::new(run, on_success, on_fail));

        self.clone()
    }
}

impl<TData: ContextData> From<JobActionsRegistryBuilder<TData>> for JobActionsRegistry<TData> {
    fn from(value: JobActionsRegistryBuilder<TData>) -> Self {
        JobActionsRegistry::new(
            value
                .inner
                .lock()
                .unwrap()
                .job_actions_map
                .drain()
                .collect(),
        )
    }
}

pub trait AddActionsRegistryService<TData: ContextData> {
    fn add_job_actions_registry<B>(&self, builder: B) -> Self
    where
        B: FnOnce(&mut JobActionsRegistryBuilder<TData>);
}

impl<TData: ContextData> AddActionsRegistryService<TData> for Services<TData> {
    fn add_job_actions_registry<B>(&self, builder: B) -> Self
    where
        B: FnOnce(&mut JobActionsRegistryBuilder<TData>),
    {
        log::debug!("adding JobActionsRegistry as a service");
        let mut b = JobActionsRegistryBuilder::default();
        builder(&mut b);
        self.add_service_unchecked(JobActionsRegistry::from(b));
        self.clone()
    }
}
