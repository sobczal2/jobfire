use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::{
    context::{JobContext, JobContextData},
    report::Report,
};

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
    fn name_dyn(&self) -> JobImplName {
        Self::name()
    }
    async fn run(&self, context: JobContext<TData>) -> super::error::Result<Report>;
    async fn on_fail(&self, context: JobContext<TData>) -> super::error::Result<()>;
    async fn on_success(&self, context: JobContext<TData>) -> super::error::Result<()>;
}
