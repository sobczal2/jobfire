use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::domain::run::job_actions::{OnFailFn, OnSuccessFn, RunFn};

use super::{context::ContextData, data::JobData};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PolicyName(String);

impl PolicyName {
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl Display for PolicyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Policy<TData: ContextData>: Send + Sync + 'static {
    fn name(&self) -> PolicyName;
    fn init(&self, data: JobData);
    fn wrap_run(&self, f: RunFn<TData>, _data: JobData) -> RunFn<TData> {
        f
    }
    fn wrap_on_fail(&self, f: OnFailFn<TData>, _data: JobData) -> OnFailFn<TData> {
        f
    }
    fn wrap_on_success(&self, f: OnSuccessFn<TData>, _data: JobData) -> OnSuccessFn<TData> {
        f
    }
}
