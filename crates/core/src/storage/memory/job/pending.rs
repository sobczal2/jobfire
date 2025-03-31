use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::{
    domain::job::pending::PendingJob,
    storage::{error::Error, job::PendingJobRepo},
};

#[derive(Default)]
pub struct MemoryPendingJobRepo {
    elements: Arc<RwLock<Vec<PendingJob>>>,
}

#[async_trait]
impl PendingJobRepo for MemoryPendingJobRepo {
    async fn get(
        &self,
        job_id: &crate::domain::job::id::JobId,
    ) -> crate::storage::error::Result<Option<PendingJob>> {
        let job = self
            .elements
            .read()
            .await
            .iter()
            .find(|job| job.job_id() == job_id)
            .cloned();
        Ok(job)
    }

    async fn add(&self, job: PendingJob) -> crate::storage::error::Result<()> {
        let existing_job = self.get(job.job_id()).await?;
        if existing_job.is_some() {
            return Err(Error::AlreadyExists);
        }

        self.elements.write().await.push(job);
        Ok(())
    }

    async fn delete(
        &self,
        job_id: &crate::domain::job::id::JobId,
    ) -> crate::storage::error::Result<PendingJob> {
        let existing_index = self
            .elements
            .read()
            .await
            .iter()
            .enumerate()
            .find(|(_, job)| job.job_id() == job_id)
            .map(|(index, _)| index);

        match existing_index {
            Some(existing_index) => {
                let job = self.elements.write().await.swap_remove(existing_index);
                Ok(job)
            }
            None => Err(Error::NotFound),
        }
    }

    async fn pop_scheduled(
        &self,
        now: DateTime<Utc>,
    ) -> crate::storage::error::Result<Option<PendingJob>> {
        let existing_index = self
            .elements
            .read()
            .await
            .iter()
            .enumerate()
            .find(|(_, job)| *job.scheduled_at() < now)
            .map(|(i, _)| i);

        match existing_index {
            Some(existing_index) => {
                let job = self.elements.write().await.swap_remove(existing_index);
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }
}
