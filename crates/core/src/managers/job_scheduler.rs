use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    domain::job::{
        self,
        id::JobId,
        pending::PendingJob,
        scheduler::{self, JobScheduler},
    },
    storage::{self, Storage},
};

#[derive(Clone)]
pub struct SimpleJobScheduler {
    storage: Storage,
}

impl SimpleJobScheduler {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl JobScheduler for SimpleJobScheduler {
    async fn schedule(&self, pending_job: &PendingJob) -> job::scheduler::Result<()> {
        if self
            .storage
            .pending_job_repo()
            .get(pending_job.id())
            .await?
            .is_some()
        {
            return Err(scheduler::Error::JobAlreadyScheduled);
        }
        self.storage.pending_job_repo().add(pending_job).await?;
        Ok(())
    }

    async fn cancel(&self, job_id: &JobId) -> job::scheduler::Result<()> {
        match self.storage.pending_job_repo().delete(job_id).await {
            Ok(_) => Ok(()),
            Err(error) => match error {
                storage::error::Error::NotFound => Err(scheduler::Error::JobNotFound),
                _ => Err(scheduler::Error::Storage(error)),
            },
        }
    }

    async fn reschedule(
        &self,
        job_id: &JobId,
        new_scheduled_at: DateTime<Utc>,
    ) -> job::scheduler::Result<()> {
        let scheduled_job = self.storage.pending_job_repo().get(job_id).await?;
        if scheduled_job.is_none() {
            return Err(scheduler::Error::JobNotFound);
        }
        let mut scheduled_job = scheduled_job.unwrap();
        scheduled_job.reschedule(new_scheduled_at);
        self.storage.pending_job_repo().delete(job_id).await?;
        self.storage.pending_job_repo().add(&scheduled_job).await?;
        Ok(())
    }
}
