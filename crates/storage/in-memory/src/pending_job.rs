use std::sync::Arc;

use chrono::Utc;
use jobfire_core::{
    async_trait,
    domain::job::{id::JobId, pending::PendingJob},
    storage::{
        error::{Error, Result},
        job::{AddJob, DeleteJob, GetJob, PendingJobRepo},
    },
};
use tokio::sync::RwLock;

use crate::{impl_add_job, impl_delete_job, impl_get_job};

#[derive(Default)]
pub(crate) struct PendingJobRepoImpl {
    elements: Arc<RwLock<Vec<PendingJob>>>,
}

impl_get_job!(PendingJobRepoImpl, PendingJob);
impl_add_job!(PendingJobRepoImpl, PendingJob);
impl_delete_job!(PendingJobRepoImpl, PendingJob);

#[async_trait]
impl PendingJobRepo for PendingJobRepoImpl {
    async fn pop_scheduled(&self) -> Result<Option<PendingJob>> {
        let existing_index = self
            .elements
            .read()
            .await
            .iter()
            .enumerate()
            .find(|(_, e)| *e.scheduled_at() < Utc::now())
            .map(|(i, _)| i);

        if existing_index.is_none() {
            return Ok(None);
        }
        let existing_index = existing_index.unwrap();

        let popped_element = self.elements.write().await.swap_remove(existing_index);

        Ok(Some(popped_element))
    }
}
