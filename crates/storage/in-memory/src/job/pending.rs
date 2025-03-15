use std::sync::{Arc, RwLock};

use chrono::Utc;
use jobfire_core::prelude::*;
use jobfire_core::storage::{Error, Result};

pub(crate) struct PendingJobRepoImpl<T: JobContext> {
    elements: Arc<RwLock<Vec<PendingJob<T>>>>,
}

impl<T: JobContext> Default for PendingJobRepoImpl<T> {
    fn default() -> Self {
        Self {
            elements: Default::default(),
        }
    }
}

#[async_trait]
impl<T: JobContext> PendingJobRepo<T> for PendingJobRepoImpl<T> {
    async fn add(&self, pending_job: PendingJob<T>) -> Result<()> {
        self.elements
            .write()
            .map_err(|e| Error::new(e.to_string()))?
            .push(pending_job);
        Ok(())
    }

    async fn find_scheduled(&self) -> Result<Option<PendingJob<T>>> {
        let elements = self
            .elements
            .read()
            .map_err(|e| Error::new(e.to_string()))?;
        let now = Utc::now();
        let job = elements.iter().find(|j| j.scheduled_at() < &now).cloned();
        Ok(job)
    }

    async fn delete(&self, id: JobId) -> Result<()> {
        let mut elements = self
            .elements
            .write()
            .map_err(|e| Error::new(e.to_string()))?;
        let to_remove_index = elements
            .iter()
            .enumerate()
            .find(|(_, j)| *j.id() == id)
            .ok_or(Error::new("element not found".to_owned()))?
            .0;
        elements.swap_remove(to_remove_index);
        Ok(())
    }
}
