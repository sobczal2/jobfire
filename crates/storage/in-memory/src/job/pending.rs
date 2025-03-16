use std::sync::{Arc, RwLock};

use chrono::Utc;
use jobfire_core::{
    async_trait,
    domain::{
        error::Error,
        job::{JobId, PendingJob},
    },
    storage::job::PendingJobRepo,
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
    async fn add(&self, pending_job: PendingJob) -> jobfire_core::storage::error::Result<()> {
        self.elements
            .write()
            .map_err(|e| jobfire_core::storage::error::Error::new(e.to_string()))?
            .push(pending_job);
        Ok(())
    }

    async fn find_scheduled(&self) -> jobfire_core::storage::error::Result<Option<PendingJob>> {
        let elements = self
            .elements
            .read()
            .map_err(|e| jobfire_core::storage::error::Error::new(e.to_string()))?;
        let now = Utc::now();
        let job = elements.iter().find(|j| j.scheduled_at() < &now).cloned();
        Ok(job)
    }

    async fn delete(&self, id: JobId) -> jobfire_core::storage::error::Result<()> {
        let mut elements = self
            .elements
            .write()
            .map_err(|e| jobfire_core::storage::error::Error::new(e.to_string()))?;
        let to_remove_index = elements
            .iter()
            .enumerate()
            .find(|(_, j)| *j.id() == id)
            .ok_or(jobfire_core::storage::error::Error::new(
                "element not found".to_owned(),
            ))?
            .0;
        elements.swap_remove(to_remove_index);
        Ok(())
    }
}
