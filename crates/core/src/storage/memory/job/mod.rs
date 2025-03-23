pub mod pending;
pub mod running;

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    domain::job::Job,
    storage::{error::Error, job::JobRepo},
};

#[derive(Default)]
pub struct MemoryJobRepo {
    elements: Arc<RwLock<Vec<Job>>>,
}

#[async_trait]
impl JobRepo for MemoryJobRepo {
    async fn get(
        &self,
        job_id: &crate::domain::job::id::JobId,
    ) -> crate::storage::error::Result<Option<Job>> {
        let job = self
            .elements
            .read()
            .await
            .iter()
            .find(|job| job.id() == job_id)
            .cloned();
        Ok(job)
    }

    async fn add(&self, job: Job) -> crate::storage::error::Result<()> {
        let existing_job = self.get(job.id()).await?;
        if existing_job.is_some() {
            return Err(Error::AlreadyExists);
        }

        self.elements.write().await.push(job);
        Ok(())
    }

    async fn delete(
        &self,
        job_id: &crate::domain::job::id::JobId,
    ) -> crate::storage::error::Result<Job> {
        let existing_index = self
            .elements
            .read()
            .await
            .iter()
            .enumerate()
            .find(|(_, job)| job.id() == job_id)
            .map(|(index, _)| index);

        match existing_index {
            Some(existing_index) => {
                let job = self.elements.write().await.swap_remove(existing_index);
                Ok(job)
            }
            None => Err(Error::NotFound),
        }
    }
}
