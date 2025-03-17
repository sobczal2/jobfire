use async_trait::async_trait;

use crate::domain::job::{FailedJob, JobId, PendingJob, RunningJob, SuccessfulJob};

use super::error::Result;

#[async_trait]
pub trait PendingJobRepo: Send + Sync {
    async fn add(&self, pending_job: &PendingJob) -> Result<()>;
    async fn pop_scheduled(&self) -> Result<Option<PendingJob>>;
}

#[async_trait]
pub trait RunningJobRepo: Send + Sync {
    async fn add(&self, running_job: &RunningJob) -> Result<()>;
    async fn pop(&self, job_id: &JobId) -> Result<RunningJob>;
}

#[async_trait]
pub trait FailedJobRepo: Send + Sync {
    async fn add(&self, failed_job: &FailedJob) -> Result<()>;
}

#[async_trait]
pub trait SuccessfulJobRepo: Send + Sync {
    async fn add(&self, successful_job: &SuccessfulJob) -> Result<()>;
}
