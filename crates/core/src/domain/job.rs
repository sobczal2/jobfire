use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_getters::Getters;
use uuid::Uuid;

#[async_trait]
pub(crate) trait JobImpl: Send + Sync {
    async fn run(&self) -> Result<Report, Error>;
    async fn on_start(&self);
    async fn on_fail(&self);
    async fn on_success(&self);
}

pub(crate) struct Error {
    message: String,
}

pub(crate) struct Report {}

pub(crate) struct Progress;

#[derive(Clone, Getters)]
pub(crate) struct PendingJob {
    id: Uuid,
    created_at: DateTime<Utc>,
    scheduled_at: DateTime<Utc>,
    r#impl: Arc<dyn JobImpl>,
}

pub(crate) struct RunningJob {
    id: Uuid,
    created_at: DateTime<Utc>,
    started_at: DateTime<Utc>,
}

impl RunningJob {
    pub fn new(id: Uuid, created_at: DateTime<Utc>, started_at: DateTime<Utc>) -> Self {
        Self {
            id,
            created_at,
            started_at,
        }
    }
}

pub(crate) struct SuccessfulJob {
    id: Uuid,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    report: Report,
}

impl SuccessfulJob {
    pub fn new(
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
        }
    }
}

pub(crate) struct FailedJob {
    id: Uuid,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    error: Error,
}
