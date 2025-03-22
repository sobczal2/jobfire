use std::sync::Arc;

use jobfire_core::{
    async_trait,
    domain::job::Job,
    storage::{
        error::Error,
        job::{AddJob, GetJob, JobRepo},
    },
};
use tokio::sync::RwLock;

#[derive(Default)]
pub(crate) struct JobRepoImpl {
    elements: Arc<RwLock<Vec<Job>>>,
}

#[async_trait]
impl GetJob<Job> for JobRepoImpl {
    async fn get(
        &self,
        job_id: &jobfire_core::domain::job::id::JobId,
    ) -> jobfire_core::storage::error::Result<Option<Job>> {
        Ok(self
            .elements
            .read()
            .await
            .iter()
            .find(|job| job.id() == job_id)
            .cloned())
    }
}

#[async_trait]
impl AddJob<Job> for JobRepoImpl {
    async fn add(&self, job: Job) -> jobfire_core::storage::error::Result<()> {
        if self.get(job.id()).await?.is_some() {
            return Err(Error::AlreadyExists);
        }

        self.elements.write().await.push(job);
        Ok(())
    }
}

#[async_trait]
impl JobRepo for JobRepoImpl {}
