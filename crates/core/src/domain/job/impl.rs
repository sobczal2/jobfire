use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::{context::JobContext, report::Report, scheduler::JobScheduler};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct JobImplName(String);

impl JobImplName {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

#[async_trait]
pub trait JobImpl<T: JobContext>:
    Serialize + DeserializeOwned + Sized + Send + Sync + 'static
{
    fn name() -> JobImplName;
    fn name_dyn(&self) -> JobImplName {
        Self::name()
    }
    async fn run(&self, context: T) -> super::error::Result<Report>;
    async fn on_fail(&self, context: T) -> super::error::Result<()>;
    async fn on_success(&self, context: T) -> super::error::Result<()>;
}
