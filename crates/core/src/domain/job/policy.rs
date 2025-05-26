use serde::{Deserialize, Serialize};

use crate::registries::job_actions::{OnFailFn, OnSuccessFn, RunFn};

use super::{context::ContextData, data::JobData};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PolicyName(String);

impl PolicyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

pub trait Policy<TData: ContextData>: Send + Sync + 'static {
    fn wrap_run(&self, f: RunFn<TData>, data: JobData) -> RunFn<TData>;
    fn wrap_on_fail(&self, f: OnFailFn<TData>, data: JobData) -> OnFailFn<TData>;
    fn wrap_on_success(&self, f: OnSuccessFn<TData>, data: JobData) -> OnSuccessFn<TData>;
}
