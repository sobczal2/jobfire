use std::sync::{Arc, RwLock};

use chrono::Utc;
use jobfire_core::{
    async_trait,
    domain::job::{id::JobId, pending::PendingJob},
    storage::{
        error::{Error, Result},
        job::PendingJobRepo,
    },
};

#[derive(Default)]
pub(crate) struct PendingJobRepoImpl {
    elements: Arc<RwLock<Vec<PendingJob>>>,
}

#[async_trait]
impl PendingJobRepo for PendingJobRepoImpl {
    async fn get(&self, job_id: &JobId) -> Result<Option<PendingJob>> {
        Ok(self
            .elements
            .read()
            .map_err(|_| Error::Internal)?
            .iter()
            .find(|e| e.job_id() == job_id)
            .cloned())
    }

    async fn add(&self, pending_job: PendingJob) -> Result<()> {
        let existing = self.get(pending_job.job_id()).await?;
        if existing.is_some() {
            return Err(Error::AlreadyExists);
        }

        self.elements
            .write()
            .map_err(|_| Error::Internal)?
            .push(pending_job.clone());
        Ok(())
    }

    async fn pop_scheduled(&self) -> Result<Option<PendingJob>> {
        let existing_index = self
            .elements
            .read()
            .map_err(|_| Error::Internal)?
            .iter()
            .enumerate()
            .find(|(_, e)| *e.scheduled_at() < Utc::now())
            .map(|(i, _)| i);

        if existing_index.is_none() {
            return Ok(None);
        }
        let existing_index = existing_index.unwrap();

        let popped_element = self
            .elements
            .write()
            .map_err(|_| Error::Internal)?
            .swap_remove(existing_index);

        Ok(Some(popped_element))
    }

    async fn delete(&self, job_id: &JobId) -> Result<()> {
        let existing_index = self
            .elements
            .read()
            .map_err(|_| Error::Internal)?
            .iter()
            .enumerate()
            .find(|(_, e)| e.job_id() == job_id)
            .map(|(i, _)| i);

        if existing_index.is_none() {
            return Err(Error::NotFound);
        }
        let existing_index = existing_index.unwrap();

        self.elements
            .write()
            .map_err(|_| Error::Internal)?
            .swap_remove(existing_index);

        Ok(())
    }
}
