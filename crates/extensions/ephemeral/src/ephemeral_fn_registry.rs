#![allow(type_alias_bounds)]

use std::{collections::HashMap, pin::Pin, sync::Arc};

use jobfire_core::{
    domain::job::{
        context::{Context, ContextData},
        error::JobResult,
        report::Report,
    },
    services::verify::VerifyService,
};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::r#impl::EphemeralJobId;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ephemeral job already added")]
    AlreadyAdded,
    #[error("ephemeral job not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

pub type EphemeralRunFn<TData: ContextData> = Arc<
    dyn Fn(Context<TData>) -> Pin<Box<dyn Future<Output = JobResult<Report>> + Send>> + Send + Sync,
>;
pub type EphemeralOnSuccessFn<TData: ContextData> =
    Arc<dyn Fn(Context<TData>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;
pub type EphemeralOnFailFn<TData: ContextData> =
    Arc<dyn Fn(Context<TData>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub struct EphemeralActions<TData: ContextData> {
    run: EphemeralRunFn<TData>,
    on_success: EphemeralOnSuccessFn<TData>,
    on_fail: EphemeralOnFailFn<TData>,
}

impl<TData: ContextData> EphemeralActions<TData> {
    pub fn new(
        run: EphemeralRunFn<TData>,
        on_success: EphemeralOnSuccessFn<TData>,
        on_fail: EphemeralOnFailFn<TData>,
    ) -> Self {
        Self {
            run,
            on_success,
            on_fail,
        }
    }
}

impl<TData: ContextData> Clone for EphemeralActions<TData> {
    fn clone(&self) -> Self {
        Self {
            run: self.run.clone(),
            on_success: self.on_success.clone(),
            on_fail: self.on_fail.clone(),
        }
    }
}

impl<TData: ContextData> EphemeralActions<TData> {
    pub async fn run(&self, context: Context<TData>) -> JobResult<Report> {
        (self.run.clone())(context).await
    }

    pub async fn on_success(&self, context: Context<TData>) {
        (self.on_success.clone())(context).await
    }

    pub async fn on_fail(&self, context: Context<TData>) {
        (self.on_fail.clone())(context).await
    }
}

pub struct EphemeralFnRegistry<TData: ContextData> {
    inner: Arc<RwLock<EphemeralFnRegistryInner<TData>>>,
}

impl<TData: ContextData> Clone for EphemeralFnRegistry<TData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TData: ContextData> Default for EphemeralFnRegistry<TData> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<TData: ContextData> VerifyService<TData> for EphemeralFnRegistry<TData> {
    fn verify(
        &self,
        _services: &jobfire_core::services::Services<TData>,
    ) -> std::result::Result<(), jobfire_core::services::verify::ServiceMissing> {
        Ok(())
    }
}

pub struct EphemeralFnRegistryInner<TData: ContextData> {
    ephemeral_actions_map: HashMap<EphemeralJobId, EphemeralActions<TData>>,
}

impl<TData: ContextData> EphemeralFnRegistry<TData> {
    pub fn new(ephemeral_actions_map: HashMap<EphemeralJobId, EphemeralActions<TData>>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(EphemeralFnRegistryInner {
                ephemeral_actions_map,
            })),
        }
    }

    pub async fn get(&self, ephemeral_job_id: &EphemeralJobId) -> Option<EphemeralActions<TData>> {
        self.inner
            .read()
            .await
            .ephemeral_actions_map
            .get(ephemeral_job_id)
            .cloned()
    }

    pub async fn get_run_fn(
        &self,
        ephemeral_job_id: &EphemeralJobId,
    ) -> Option<EphemeralRunFn<TData>> {
        self.get(ephemeral_job_id).await.map(|ja| ja.run.clone())
    }

    pub async fn get_on_success_fn(
        &self,
        ephemeral_job_id: &EphemeralJobId,
    ) -> Option<EphemeralOnSuccessFn<TData>> {
        self.get(ephemeral_job_id)
            .await
            .map(|ja| ja.on_success.clone())
    }

    pub async fn get_on_fail_fn(
        &self,
        ephemeral_job_id: &EphemeralJobId,
    ) -> Option<EphemeralOnFailFn<TData>> {
        self.get(ephemeral_job_id)
            .await
            .map(|ja| ja.on_fail.clone())
    }

    pub async fn add(
        &self,
        ephemeral_job_id: &EphemeralJobId,
        ephemeral_actions: EphemeralActions<TData>,
    ) -> Result<()> {
        let mut guard = self.inner.write().await;
        match guard.ephemeral_actions_map.contains_key(ephemeral_job_id) {
            true => Err(Error::AlreadyAdded),
            false => {
                guard
                    .ephemeral_actions_map
                    .insert(*ephemeral_job_id, ephemeral_actions);
                Ok(())
            }
        }
    }

    pub async fn remove(&self, ephemeral_job_id: &EphemeralJobId) -> Result<()> {
        if self
            .inner
            .write()
            .await
            .ephemeral_actions_map
            .remove(ephemeral_job_id)
            .is_some()
        {
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }
}
