#![allow(type_alias_bounds)]

use std::{collections::HashMap, sync::Arc};

use thiserror::Error;

use crate::{
    domain::{
        job::{
            context::ContextData,
            r#impl::{JobImpl, JobImplName},
        },
        run::job_actions::{JobActions, OnFailFn, OnSuccessFn, RunFn},
    },
    services::{
        verify::{ServiceMissing, VerifyService},
        Services,
    },
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to retrieve fn")]
    FnNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct JobActionsRegistry<TData: ContextData> {
    job_actions: Arc<HashMap<JobImplName, JobActions<TData>>>,
}

impl<TData: ContextData> Clone for JobActionsRegistry<TData> {
    fn clone(&self) -> Self {
        Self {
            job_actions: self.job_actions.clone(),
        }
    }
}

impl<TData: ContextData> JobActionsRegistry<TData> {
    pub fn get(&self, job_impl_name: &JobImplName) -> Option<JobActions<TData>> {
        self.job_actions.get(job_impl_name).cloned()
    }

    pub fn get_run_fn(&self, job_impl_name: &JobImplName) -> Option<RunFn<TData>> {
        self.get(job_impl_name).map(|ja| ja.get_run_fn())
    }

    pub fn get_on_success_fn(&self, job_impl_name: &JobImplName) -> Option<OnSuccessFn<TData>> {
        self.get(job_impl_name).map(|ja| ja.get_on_success_fn())
    }

    pub fn get_on_fail_fn(&self, job_impl_name: &JobImplName) -> Option<OnFailFn<TData>> {
        self.get(job_impl_name).map(|ja| ja.get_on_fail_fn())
    }
}

impl<TData: ContextData> VerifyService for JobActionsRegistry<TData> {
    fn verify(&self, _services: &Services) -> std::result::Result<(), ServiceMissing> {
        Ok(())
    }
}

pub struct JobActionsRegistryBuilder<TData: ContextData> {
    job_actions: HashMap<JobImplName, JobActions<TData>>,
}

impl<TData: ContextData> Default for JobActionsRegistryBuilder<TData> {
    fn default() -> Self {
        Self {
            job_actions: Default::default(),
        }
    }
}

impl<TData: ContextData> JobActionsRegistryBuilder<TData> {
    pub fn register<TJobImpl: JobImpl<TData>>(&mut self) {
        self.job_actions
            .insert(TJobImpl::name(), JobActions::from_job_impl::<TJobImpl>());
    }

    pub fn build(self) -> JobActionsRegistry<TData> {
        JobActionsRegistry {
            job_actions: Arc::new(self.job_actions),
        }
    }
}
