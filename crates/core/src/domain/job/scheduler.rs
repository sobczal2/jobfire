use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::storage;

use super::{id::JobId, pending::PendingJob};

#[derive(Error, Debug)]
pub enum Error {
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
    #[error("job not found")]
    JobNotFound,
    #[error("job already scheduled")]
    JobAlreadyScheduled,
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait JobScheduler: Send + Sync {
    async fn schedule(&self, pending_job: &PendingJob) -> Result<()>;
    async fn cancel(&self, job_id: &JobId) -> Result<()>;
    async fn reschedule(&self, job_id: &JobId, new_scheduled_at: DateTime<Utc>) -> Result<()>;
}
