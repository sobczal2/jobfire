use std::hash::Hash;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, to_value};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct JobImplName(String);

impl JobImplName {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct JobId(Uuid);

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

pub trait JobContext: Sized + Clone + Send + Sync + 'static {}

#[async_trait]
pub trait JobImpl<T: JobContext>:
    Serialize + DeserializeOwned + Sized + Send + Sync + 'static
{
    fn name() -> JobImplName;
    fn name_dyn(&self) -> JobImplName {
        Self::name()
    }
    async fn run(&self, context: T) -> Result<Report, JobError>;
    async fn on_fail(&self, context: T);
    async fn on_success(&self, context: T);
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JobError {
    message: String,
}

impl JobError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Report {}

impl Report {
    pub fn new() -> Self {
        Report {}
    }
}

pub struct Progress;

#[derive(Clone, Getters, Serialize, Deserialize, Hash)]
#[getset(get = "pub")]
pub struct PendingJob {
    id: JobId,
    created_at: DateTime<Utc>,
    scheduled_at: DateTime<Utc>,
    r#impl: Value,
    impl_name: JobImplName,
}

impl PendingJob {
    pub(crate) fn new<TJobContext: JobContext>(
        id: JobId,
        created_at: DateTime<Utc>,
        scheduled_at: DateTime<Utc>,
        r#impl: impl JobImpl<TJobContext>,
    ) -> super::error::Result<Self> {
        let impl_name = r#impl.name_dyn();
        Ok(Self {
            id,
            created_at,
            scheduled_at,
            r#impl: to_value(r#impl).map_err(|_| super::error::Error::SerializationFailed)?,
            impl_name,
        })
    }

    pub fn new_at<TJobContext: JobContext>(
        scheduled_at: DateTime<Utc>,
        r#impl: impl JobImpl<TJobContext>,
    ) -> super::error::Result<Self> {
        Self::new(JobId::new(), Utc::now(), scheduled_at, r#impl)
    }

    pub fn build_impl<TJobContext: JobContext, TJobImpl: JobImpl<TJobContext>>(
        &self,
    ) -> super::error::Result<TJobImpl> {
        let job_impl = serde_json::from_value::<TJobImpl>(self.r#impl.clone())
            .map_err(|_| super::error::Error::BuildJobImplFailed)?;
        Ok(job_impl)
    }
}

#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct RunningJob {
    id: JobId,
    created_at: DateTime<Utc>,
    started_at: DateTime<Utc>,
}

impl RunningJob {
    pub fn new(id: JobId, created_at: DateTime<Utc>, started_at: DateTime<Utc>) -> Self {
        Self {
            id,
            created_at,
            started_at,
        }
    }
}

#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct SuccessfulJob {
    id: JobId,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    report: Report,
}

impl SuccessfulJob {
    pub fn new(
        id: JobId,
        created_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        report: Report,
    ) -> Self {
        Self {
            id,
            created_at,
            finished_at,
            report,
        }
    }
}

#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct FailedJob {
    id: JobId,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    error: JobError,
}

impl FailedJob {
    pub fn new(
        id: JobId,
        created_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        error: JobError,
    ) -> Self {
        Self {
            id,
            created_at,
            finished_at,
            error,
        }
    }
}
