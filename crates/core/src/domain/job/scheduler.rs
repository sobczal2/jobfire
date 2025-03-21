use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::storage;

use super::{Job, id::JobId};

#[derive(Error, Debug)]
pub enum Error {
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
    #[error("job not found")]
    JobNotFound,
    #[error("already scheduled")]
    AlreadyScheduled,
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait JobScheduler: Send + Sync {
    async fn schedule(&self, job: Job, scheduled_at: DateTime<Utc>) -> Result<()>;
    async fn cancel(&self, job_id: &JobId) -> Result<()>;
    async fn reschedule(&self, job_id: &JobId, new_scheduled_at: DateTime<Utc>) -> Result<()>;
}
