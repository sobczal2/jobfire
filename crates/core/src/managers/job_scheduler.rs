use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{
    domain::job::{JobId, JobScheduler, PendingJob},
    storage::{self, Storage},
};

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

pub struct JobSchedulerImpl {
    storage: Storage,
}

impl JobSchedulerImpl {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl JobScheduler for JobSchedulerImpl {
    async fn schedule(&self, pending_job: &PendingJob) -> Result<()> {
        if self
            .storage
            .pending_job_repo()
            .get(pending_job.id())
            .await?
            .is_some()
        {
            return Err(Error::JobAlreadyScheduled);
        }
        self.storage.pending_job_repo().add(pending_job).await?;
        Ok(())
    }

    async fn cancel(&self, job_id: &JobId) -> Result<()> {
        match self.storage.pending_job_repo().delete(job_id).await {
            Ok(_) => Ok(()),
            Err(error) => match error {
                storage::error::Error::NotFound => Err(Error::JobNotFound),
                _ => Err(Error::Storage(error)),
            },
        }
    }

    async fn reschedule(&self, job_id: &JobId, new_scheduled_at: DateTime<Utc>) -> Result<()> {
        let scheduled_job = self.storage.pending_job_repo().get(job_id).await?;
        if scheduled_job.is_none() {
            return Err(Error::JobNotFound);
        }
        let mut scheduled_job = scheduled_job.unwrap();
        scheduled_job.reschedule(new_scheduled_at);
        self.storage.pending_job_repo().delete(job_id).await?;
        self.storage.pending_job_repo().add(&scheduled_job).await?;
        Ok(())
    }
}
