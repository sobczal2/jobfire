use async_trait::async_trait;

use super::error::Result;
use crate::domain::{FailedJob, JobContext, JobId, PendingJob, RunningJob, SuccessfulJob};

#[async_trait]
pub trait PendingJobRepo<T: JobContext>: Send + Sync {
    async fn add(&self, pending_job: PendingJob<T>) -> Result<()>;
    async fn find_scheduled(&self) -> Result<Option<PendingJob<T>>>;
    async fn delete(&self, id: JobId) -> Result<()>;
}

#[async_trait]
pub trait RunningJobRepo<T: JobContext>: Send + Sync {
    async fn add(&self, running_job: RunningJob<T>) -> Result<()>;
}

#[async_trait]
pub trait FailedJobRepo<T: JobContext>: Send + Sync {
    async fn add(&self, failed_job: FailedJob<T>) -> Result<()>;
}

#[async_trait]
pub trait SuccessfulJobRepo<T: JobContext>: Send + Sync {
    async fn add(&self, successful_job: SuccessfulJob<T>) -> Result<()>;
}
