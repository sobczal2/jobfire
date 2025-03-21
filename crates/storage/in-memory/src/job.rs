use std::sync::{Arc, RwLock};

use jobfire_core::{
    async_trait,
    domain::job::Job,
    storage::{error::Error, job::JobRepo},
};

#[derive(Default)]
pub(crate) struct JobRepoImpl {
    elements: Arc<RwLock<Vec<Job>>>,
}

#[async_trait]
impl JobRepo for JobRepoImpl {
    async fn get(
        &self,
        job_id: &jobfire_core::domain::job::id::JobId,
    ) -> jobfire_core::storage::error::Result<Option<Job>> {
        Ok(self
            .elements
            .read()
            .map_err(|_| Error::Internal)?
            .iter()
            .find(|e| e.id() == job_id)
            .cloned())
    }

    async fn add(&self, job: Job) -> jobfire_core::storage::error::Result<()> {
        let existing = self.get(job.id()).await?;
        if existing.is_some() {
            return Err(Error::AlreadyExists);
        }

        self.elements
            .write()
            .map_err(|_| Error::Internal)?
            .push(job.clone());

        Ok(())
    }
}
