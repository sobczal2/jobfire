use crate::{
    domain::{
        job::{
            context::ContextData,
            data::JobData,
            policy::{Policy, PolicyName},
        },
        run::job_actions::RunFn,
    },
    services::{
        verify::{ServiceMissing, VerifyService},
        Services,
    },
};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to retrieve fn")]
    PolicyNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct PolicyRegistry<TData: ContextData> {
    policies: Arc<HashMap<PolicyName, Box<dyn Policy<TData>>>>,
}

impl<TData: ContextData> Clone for PolicyRegistry<TData> {
    fn clone(&self) -> Self {
        Self {
            policies: self.policies.clone(),
        }
    }
}

impl<TData: ContextData> PolicyRegistry<TData> {
    pub fn wrap_run(
        &self,
        name: PolicyName,
        f: RunFn<TData>,
        data: JobData,
    ) -> Result<RunFn<TData>> {
        Ok(self
            .policies
            .get(&name)
            .ok_or(Error::PolicyNotFound)?
            .wrap_run(f, data))
    }
}

impl<TData: ContextData> VerifyService for PolicyRegistry<TData> {
    fn verify(&self, _services: &Services) -> std::result::Result<(), ServiceMissing> {
        Ok(())
    }
}

pub struct PolicyRegistryBuilder<TData: ContextData> {
    policies: HashMap<PolicyName, Box<dyn Policy<TData>>>,
}

impl<TData: ContextData> Default for PolicyRegistryBuilder<TData> {
    fn default() -> Self {
        Self {
            policies: Default::default(),
        }
    }
}

impl<TData: ContextData> PolicyRegistryBuilder<TData> {
    pub fn register(&mut self, policy: impl Policy<TData>) {
        self.policies.insert(policy.name(), Box::new(policy));
    }

    pub fn build(self) -> PolicyRegistry<TData> {
        PolicyRegistry {
            policies: Arc::new(self.policies),
        }
    }
}
