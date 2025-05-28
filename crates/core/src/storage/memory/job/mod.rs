pub mod pending;
pub mod running;

use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use crate::{
    domain::job::{Job, data::JobData, id::JobId},
    storage::{error::Error, job::JobRepo},
};

#[derive(Default)]
pub struct MemoryJobRepo {
    elements: Arc<RwLock<Vec<Job>>>,
}

#[async_trait]
impl JobRepo for MemoryJobRepo {
    async fn get(&self, job_id: &JobId) -> crate::storage::error::Result<Option<Job>> {
        let job = self
            .elements
            .read()
            .unwrap()
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

        self.elements.write().unwrap().push(job);
        Ok(())
    }

    async fn delete(&self, job_id: &JobId) -> crate::storage::error::Result<Job> {
        let existing_index = self
            .elements
            .read()
            .unwrap()
            .iter()
            .enumerate()
            .find(|(_, job)| job.id() == job_id)
            .map(|(index, _)| index);

        match existing_index {
            Some(existing_index) => {
                let job = self.elements.write().unwrap().swap_remove(existing_index);
                Ok(job)
            }
            None => Err(Error::NotFound),
        }
    }

    async fn update(&self, job_id: &JobId, data: JobData) -> crate::storage::error::Result<()> {
        let mut elements = self.elements.write().unwrap();
        let job = elements.iter_mut().find(|job| job.id() == job_id);
        if let Some(job) = job {
            job.update_data(data);
        }

        Ok(())
    }
}
