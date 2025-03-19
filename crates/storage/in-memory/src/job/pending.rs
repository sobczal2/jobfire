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

pub(crate) struct PendingJobRepoImpl {
    elements: Arc<RwLock<Vec<PendingJob>>>,
}

impl Default for PendingJobRepoImpl {
    fn default() -> Self {
        Self {
            elements: Default::default(),
        }
    }
}

#[async_trait]
impl PendingJobRepo for PendingJobRepoImpl {
    async fn get(&self, job_id: &JobId) -> Result<Option<PendingJob>> {
        Ok(self
            .elements
            .read()
            .map_err(|_| Error::Internal)?
            .iter()
            .find(|e| e.id() == job_id)
            .cloned())
    }

    async fn add(&self, pending_job: &PendingJob) -> Result<()> {
        let existing = self.get(pending_job.id()).await?;
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
        Ok(self
            .elements
            .read()
            .map_err(|_| Error::Internal)?
            .iter()
            .find(|e| *e.scheduled_at() < Utc::now())
            .cloned())
    }

    async fn delete(&self, job_id: &JobId) -> Result<()> {
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

        self.elements
            .write()
            .map_err(|_| Error::Internal)?
            .swap_remove(existing_index);

        Ok(())
    }
}
