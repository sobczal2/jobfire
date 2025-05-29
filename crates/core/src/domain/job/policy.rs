use super::context::ContextData;
use crate::domain::run::job_actions::{OnFailFn, OnSuccessFn, RunFn};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, to_value, Value};
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, RwLock},
};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policies {
    names: Vec<PolicyName>,
    data: PolicyData,
}

impl Policies {
    pub fn new(names: Vec<PolicyName>, data: PolicyData) -> Self {
        Self { names, data }
    }

    pub fn names(&self) -> &Vec<PolicyName> {
        &self.names
    }

    pub fn data(&self) -> PolicyData {
        self.data.clone()
    }
}

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
    fn init(&self, _data: PolicyData) {}
    fn wrap_run(&self, f: RunFn<TData>, _data: PolicyData) -> RunFn<TData> {
        f
    }
    fn wrap_on_fail(&self, f: OnFailFn<TData>, _data: PolicyData) -> OnFailFn<TData> {
        f
    }
    fn wrap_on_success(&self, f: OnSuccessFn<TData>, _data: PolicyData) -> OnSuccessFn<TData> {
        f
    }
}

#[derive(Default, Debug)]
pub struct PolicyData {
    inner: Arc<RwLock<JobDataInner>>,
}

impl Serialize for PolicyData {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = self.inner.read().map_err(serde::ser::Error::custom)?;
        data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PolicyData {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = JobDataInner::deserialize(deserializer)?;
        Ok(PolicyData {
            inner: Arc::new(RwLock::new(value)),
        })
    }
}

impl Clone for PolicyData {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct JobDataInner {
    values: HashMap<String, Value>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to cast")]
    CastError,
}

pub type Result<T> = std::result::Result<T, Error>;

impl PolicyData {
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match self.inner.read().unwrap().values.get(key) {
            Some(v) => Ok(Some(
                from_value::<T>(v.clone()).map_err(|_| Error::CastError)?,
            )),
            None => Ok(None),
        }
    }

    pub fn set<T: Serialize>(&self, key: &str, value: T) -> Result<()> {
        self.inner.write().unwrap().values.insert(
            key.to_owned(),
            to_value(value).map_err(|_| Error::CastError)?,
        );

        Ok(())
    }
}
