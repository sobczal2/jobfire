use std::sync::{Arc, RwLock};

use jobfire_core::{
    async_trait,
    domain::job::{id::JobId, running::RunningJob},
    storage::{
        self,
        error::{Error, Result},
        job::RunningJobRepo,
    },
};

#[derive(Default)]
pub(crate) struct RunningJobRepoImpl {
    elements: Arc<RwLock<Vec<RunningJob>>>,
}

#[async_trait]
impl RunningJobRepo for RunningJobRepoImpl {
    async fn get(&self, job_id: &JobId) -> Result<Option<RunningJob>> {
        Ok(self
            .elements
            .read()
            .map_err(|_| Error::Internal)?
            .iter()
            .find(|e| e.id() == job_id)
            .cloned())
    }

    async fn add(&self, running_job: &RunningJob) -> Result<()> {
        let existing = self.get(running_job.id()).await?;
        if existing.is_some() {
            return Err(storage::error::Error::AlreadyExists);
        }

        self.elements
            .write()
            .map_err(|_| storage::error::Error::Internal)?
            .push(running_job.clone());

        Ok(())
    }

    async fn pop(&self, job_id: &JobId) -> Result<RunningJob> {
        let existing_index = self
            .elements
            .read()
            .map_err(|_| Error::Internal)?
            .iter()
            .enumerate()
            .find(|(_, e)| e.id() == job_id)
            .map(|(i, _)| i);

        if existing_index.is_none() {
            return Err(Error::NotFound);
        }
        let existing_index = existing_index.unwrap();

        let existing_element = self
            .elements
            .write()
            .map_err(|_| Error::Internal)?
            .swap_remove(existing_index);

        Ok(existing_element)
    }
}
