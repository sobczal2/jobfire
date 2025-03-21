use std::sync::{Arc, RwLock};

use jobfire_core::{
    async_trait,
    domain::job::{id::JobId, successful::SuccessfulJob},
    storage::{
        error::{Error, Result},
        job::SuccessfulJobRepo,
    },
};

#[derive(Default)]
pub(crate) struct SuccessfulJobRepoImpl {
    elements: Arc<RwLock<Vec<SuccessfulJob>>>,
}

#[async_trait]
impl SuccessfulJobRepo for SuccessfulJobRepoImpl {
    async fn get(&self, job_id: &JobId) -> Result<Option<SuccessfulJob>> {
        Ok(self
            .elements
            .read()
            .map_err(|_| Error::Internal)?
            .iter()
            .find(|e| e.id() == job_id)
            .cloned())
    }

    async fn add(&self, successful_job: &SuccessfulJob) -> Result<()> {
        let existing = self.get(successful_job.id()).await?;
        if existing.is_some() {
            return Err(Error::AlreadyExists);
        }

        self.elements
            .write()
            .map_err(|_| Error::Internal)?
            .push(successful_job.clone());

        Ok(())
    }
}
