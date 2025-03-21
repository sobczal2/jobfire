use std::sync::{Arc, RwLock};

use jobfire_core::{
    async_trait,
    domain::job::{failed::FailedJob, id::JobId},
    storage::{
        error::{Error, Result},
        job::FailedJobRepo,
    },
};

#[derive(Default)]
pub(crate) struct FailedJobRepoImpl {
    elements: Arc<RwLock<Vec<FailedJob>>>,
}

#[async_trait]
impl FailedJobRepo for FailedJobRepoImpl {
    async fn get(&self, job_id: &JobId) -> Result<Option<FailedJob>> {
        Ok(self
            .elements
            .read()
            .map_err(|_| Error::Internal)?
            .iter()
            .find(|e| e.job_id() == job_id)
            .cloned())
    }

    async fn add(&self, failed_job: FailedJob) -> Result<()> {
        let existing = self.get(failed_job.job_id()).await?;
        if existing.is_some() {
            return Err(Error::AlreadyExists);
        }

        self.elements
            .write()
            .map_err(|_| Error::Internal)?
            .push(failed_job);

        Ok(())
    }
}
