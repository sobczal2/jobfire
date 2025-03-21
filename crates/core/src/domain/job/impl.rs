use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use thiserror::Error;

use super::{
    context::{JobContext, JobContextData},
    report::Report,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("deserialization failed")]
    DeserializationFailed,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct JobImplName(String);

impl JobImplName {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

#[async_trait]
pub trait JobImpl<TData: JobContextData>:
    Serialize + DeserializeOwned + Sized + Send + Sync + 'static
{
    fn name() -> JobImplName;
    async fn run(&self, context: JobContext<TData>) -> super::error::Result<Report>;
    async fn on_fail(&self, context: JobContext<TData>) -> super::error::Result<()>;
    async fn on_success(&self, context: JobContext<TData>) -> super::error::Result<()>;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SerializedJobImpl {
    inner: Arc<SerializedJobImplInner>,
}

impl SerializedJobImpl {
    pub fn deserialize<TData: JobContextData, TJobImpl: JobImpl<TData>>(&self) -> Result<TJobImpl> {
        serde_json::from_value(self.inner.value.clone()).map_err(|_| Error::DeserializationFailed)
    }

    pub fn name(&self) -> &JobImplName {
        &self.inner.name
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SerializedJobImplInner {
    name: JobImplName,
    value: Value,
}
