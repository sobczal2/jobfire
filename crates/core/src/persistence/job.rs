use async_trait::async_trait;
use uuid::Uuid;

use super::error::Result;
use crate::domain::job::{FailedJob, PendingJob, RunningJob, SuccessfulJob};

#[async_trait]
pub trait PendingJobRepo: Send + Sync {
    async fn add(&self, pending_job: PendingJob) -> Result<()>;
    async fn first_scheduled(&self) -> Result<Option<PendingJob>>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait RunningJobRepo: Send + Sync {
    async fn add(&self, running_job: RunningJob) -> Result<()>;
}

#[async_trait]
pub trait FailedJobRepo: Send + Sync {
    async fn add(&self, failed_job: FailedJob) -> Result<()>;
}

#[async_trait]
pub trait SuccessfullJobRepo: Send + Sync {
    async fn add(&self, successful_job: SuccessfulJob) -> Result<()>;
}
