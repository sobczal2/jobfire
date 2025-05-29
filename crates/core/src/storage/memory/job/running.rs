use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    domain::job::running::RunningJob,
    storage::{error::Error, job::RunningJobRepo},
};

#[derive(Default)]
pub struct MemoryRunningJobRepo {
    elements: Arc<RwLock<Vec<RunningJob>>>,
}

#[async_trait]
impl RunningJobRepo for MemoryRunningJobRepo {
    async fn get(
        &self,
        job_id: &crate::domain::job::id::JobId,
    ) -> crate::storage::error::Result<Option<crate::domain::job::running::RunningJob>> {
        let job = self
            .elements
            .read()
            .await
            .iter()
            .find(|job| job.job_id() == *job_id)
            .cloned();
        Ok(job)
    }

    async fn add(&self, job: RunningJob) -> crate::storage::error::Result<()> {
        let existing_job = self.get(&job.job_id()).await?;
        if existing_job.is_some() {
            return Err(Error::AlreadyExists);
        }

        self.elements.write().await.push(job);
        Ok(())
    }

    async fn delete(
        &self,
        job_id: &crate::domain::job::id::JobId,
    ) -> crate::storage::error::Result<RunningJob> {
        let existing_index = self
            .elements
            .read()
            .await
            .iter()
            .enumerate()
            .find(|(_, job)| job.job_id() == *job_id)
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
