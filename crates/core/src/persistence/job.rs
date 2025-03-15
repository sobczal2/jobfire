use async_trait::async_trait;
use uuid::Uuid;

use super::error::Result;
use crate::domain::job::{Context, FailedJob, PendingJob, RunningJob, SuccessfulJob};

#[async_trait]
pub trait PendingJobRepo<T: Context>: Send + Sync {
    async fn add(&self, pending_job: PendingJob<T>) -> Result<()>;
    async fn find_scheduled(&self) -> Result<Option<PendingJob<T>>>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait RunningJobRepo<T: Context>: Send + Sync {
    async fn add(&self, running_job: RunningJob<T>) -> Result<()>;
}

#[async_trait]
pub trait FailedJobRepo<T: Context>: Send + Sync {
    async fn add(&self, failed_job: FailedJob<T>) -> Result<()>;
}

#[async_trait]
pub trait SuccessfullJobRepo<T: Context>: Send + Sync {
    async fn add(&self, successful_job: SuccessfulJob<T>) -> Result<()>;
}
