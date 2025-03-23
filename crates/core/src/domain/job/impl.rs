use super::{
    context::{Context, ContextData},
    error::JobResult,
    report::Report,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("deserialization failed")]
    DeserializationFailed,
    #[error("serialization failed")]
    SerializationFailed,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct JobImplName(String);

impl JobImplName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

#[async_trait]
pub trait JobImpl<TData: ContextData>:
    Serialize + DeserializeOwned + Sized + Send + Sync + 'static
{
    fn name() -> JobImplName;
    async fn run(&self, context: Context<TData>) -> JobResult<Report>;
    async fn on_fail(&self, context: Context<TData>);
    async fn on_success(&self, context: Context<TData>);
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SerializedJobImpl {
    inner: Arc<SerializedJobImplInner>,
}

impl SerializedJobImpl {
    pub fn new(name: JobImplName, value: Value) -> Self {
        Self {
            inner: Arc::new(SerializedJobImplInner { name, value }),
        }
    }

    pub fn from_job_impl<TData: ContextData, TJobImpl: JobImpl<TData>>(
        value: TJobImpl,
    ) -> Result<Self> {
        let name = TJobImpl::name();
        Ok(SerializedJobImpl::new(
            name,
            serde_json::to_value(value).map_err(|_| Error::SerializationFailed)?,
        ))
    }

    pub fn deserialize<TData: ContextData, TJobImpl: JobImpl<TData>>(&self) -> Result<TJobImpl> {
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
