use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use getset::Getters;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobId(Uuid);

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

pub trait JobContext: Sized + Clone + Send + Sync + 'static {}

#[async_trait]
pub trait JobImpl<T: JobContext>: Send + Sync {
    async fn run(&self, context: T) -> Result<Report, Error>;
    async fn on_fail(&self, context: T);
    async fn on_success(&self, context: T);
}

pub struct Error {
    message: String,
}

pub struct Report {}

impl Report {
    pub fn new() -> Self {
        Report {}
    }
}

pub struct Progress;

#[derive(Clone, Getters)]
#[getset(get = "pub")]
pub struct PendingJob<T: JobContext> {
    id: JobId,
    created_at: DateTime<Utc>,
    scheduled_at: DateTime<Utc>,
    r#impl: Arc<dyn JobImpl<T>>,
}

impl<T: JobContext> PendingJob<T> {
    pub(crate) fn new(
        id: JobId,
        created_at: DateTime<Utc>,
        scheduled_at: DateTime<Utc>,
        r#impl: Arc<dyn JobImpl<T>>,
    ) -> Self {
        Self {
            id,
            created_at,
            scheduled_at,
            r#impl,
        }
    }

    pub fn new_at(scheduled_at: DateTime<Utc>, r#impl: Arc<dyn JobImpl<T>>) -> Self {
        Self::new(JobId::new(), Utc::now(), scheduled_at, r#impl)
    }
}

pub struct RunningJob<T: JobContext> {
    id: JobId,
    created_at: DateTime<Utc>,
    started_at: DateTime<Utc>,
    phantom_data: PhantomData<T>,
}

impl<T: JobContext> RunningJob<T> {
    pub fn new(id: JobId, created_at: DateTime<Utc>, started_at: DateTime<Utc>) -> Self {
        Self {
            id,
            created_at,
            started_at,
            phantom_data: PhantomData,
        }
    }
}

pub struct SuccessfulJob<T: JobContext> {
    id: JobId,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    report: Report,
    phantom_data: PhantomData<T>,
}

impl<T: JobContext> SuccessfulJob<T> {
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
            phantom_data: PhantomData,
        }
    }
}

pub struct FailedJob<T: JobContext> {
    id: JobId,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    error: Error,
    phantom_data: PhantomData<T>,
}

impl<T: JobContext> FailedJob<T> {
    pub fn new(
        id: JobId,
        created_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        error: Error,
    ) -> Self {
        Self {
            id,
            created_at,
            finished_at,
            error,
            phantom_data: PhantomData,
        }
    }
}
