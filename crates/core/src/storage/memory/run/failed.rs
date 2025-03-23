use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    domain::run::{self, failed::FailedRun},
    storage::{error::Error, run::FailedRunRepo},
};

#[derive(Default)]
pub struct MemoryFailedRunRepo {
    elements: Arc<RwLock<Vec<FailedRun>>>,
}

#[async_trait]
impl FailedRunRepo for MemoryFailedRunRepo {
    async fn get_by_run_id(
        &self,
        run_id: &crate::domain::run::id::RunId,
    ) -> crate::storage::error::Result<Option<FailedRun>> {
        let job = self
            .elements
            .read()
            .await
            .iter()
            .find(|job| job.run_id() == run_id)
            .cloned();
        Ok(job)
    }

    async fn add(&self, run: FailedRun) -> crate::storage::error::Result<()> {
        let existing_job = self.get_by_run_id(run.run_id()).await?;
        if existing_job.is_some() {
            return Err(Error::AlreadyExists);
        }

        self.elements.write().await.push(run);
        Ok(())
    }
}
