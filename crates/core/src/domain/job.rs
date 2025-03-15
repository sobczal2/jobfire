use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use getset::Getters;
use uuid::Uuid;

pub(crate) trait Context: Sized + Clone + Send + Sync + 'static {}

#[async_trait]
pub(crate) trait JobImpl<T: Context>: Send + Sync {
    async fn run(&self, contaxt: T) -> Result<Report, Error>;
    async fn on_fail(&self, context: T);
    async fn on_success(&self, context: T);
}

pub(crate) struct Error {
    message: String,
}

pub(crate) struct Report {}

pub(crate) struct Progress;

#[derive(Clone, Getters)]
#[getset(get = "pub")]
pub(crate) struct PendingJob<T: Context> {
    id: Uuid,
    created_at: DateTime<Utc>,
    scheduled_at: DateTime<Utc>,
    r#impl: Arc<dyn JobImpl<T>>,
}

impl<T: Context> PendingJob<T> {
    pub(crate) fn new(
        id: Uuid,
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
}

pub(crate) struct RunningJob<T: Context> {
    id: Uuid,
    created_at: DateTime<Utc>,
    started_at: DateTime<Utc>,
    phantom_data: PhantomData<T>,
}

impl<T: Context> RunningJob<T> {
    pub(crate) fn new(id: Uuid, created_at: DateTime<Utc>, started_at: DateTime<Utc>) -> Self {
        Self {
            id,
            created_at,
            started_at,
            phantom_data: PhantomData,
        }
    }
}

pub(crate) struct SuccessfulJob<T: Context> {
    id: Uuid,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    report: Report,
    phantom_data: PhantomData<T>,
}

impl<T: Context> SuccessfulJob<T> {
    pub(crate) fn new(
        id: Uuid,
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

pub(crate) struct FailedJob<T: Context> {
    id: Uuid,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    error: Error,
    phantom_data: PhantomData<T>,
}

impl<T: Context> FailedJob<T> {
    pub(crate) fn new(
        id: Uuid,
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
