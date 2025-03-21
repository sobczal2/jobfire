use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    domain::job::{
        self,
        context::{JobContext, JobContextData},
        r#impl::{JobImpl, JobImplName, SerializedJobImpl},
    },
    registries::job_actions::{JobActions, JobActionsRegistry, OnFailFn, OnSuccessFn, RunFn},
};

use super::Builder;

pub struct JobActionsRegistryBuilder<TData: JobContextData> {
    inner: Arc<Mutex<JobActionsRegistryBuilderInner<TData>>>,
}

impl<TData: JobContextData> Clone for JobActionsRegistryBuilder<TData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TData: JobContextData> Default for JobActionsRegistryBuilder<TData> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

pub struct JobActionsRegistryBuilderInner<TData: JobContextData> {
    job_actions_map: HashMap<JobImplName, JobActions<TData>>,
}

impl<TData: JobContextData> Default for JobActionsRegistryBuilderInner<TData> {
    fn default() -> Self {
        Self {
            job_actions_map: Default::default(),
        }
    }
}

impl<TData: JobContextData> Builder<JobActionsRegistry<TData>>
    for JobActionsRegistryBuilder<TData>
{
    fn build(self) -> super::Result<JobActionsRegistry<TData>> {
        let inner = self.inner.lock().unwrap();
        Ok(JobActionsRegistry::new(inner.job_actions_map.clone()))
    }
}

impl<TData: JobContextData> JobActionsRegistryBuilder<TData> {
    pub fn register<TJobImpl: JobImpl<TData>>(&mut self) {
        let run: RunFn<TData> = Arc::new(
            |serialized_job_impl: SerializedJobImpl, job_context: JobContext<TData>| {
                Box::pin(async move {
                    let job_impl = serialized_job_impl
                        .deserialize::<TData, TJobImpl>()
                        .map_err(|_| job::error::Error::JobImplBuildFailed);
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
            |serialized_job_impl: SerializedJobImpl, job_context: JobContext<TData>| {
                Box::pin(async move {
                    let job_impl = serialized_job_impl
                        .deserialize::<TData, TJobImpl>()
                        .map_err(|_| job::error::Error::JobImplBuildFailed);
                    match job_impl {
                        Ok(job_impl) => job_impl.on_success(job_context).await,
                        Err(e) => {
                            log::error!("failed to run on_success job action");
                            Err(e)
                        }
                    }
                })
            },
        );

        let on_fail: OnFailFn<TData> = Arc::new(
            |serialized_job_impl: SerializedJobImpl, job_context: JobContext<TData>| {
                Box::pin(async move {
                    let job_impl = serialized_job_impl
                        .deserialize::<TData, TJobImpl>()
                        .map_err(|_| job::error::Error::JobImplBuildFailed);
                    match job_impl {
                        Ok(job_impl) => job_impl.on_fail(job_context).await,
                        Err(e) => {
                            log::error!("failed to run on_fail job action");
                            Err(e)
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
    }
}
