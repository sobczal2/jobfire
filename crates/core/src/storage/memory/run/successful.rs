use crate::{
    domain::run::successful::SuccessfulRun,
    storage::{error::Error, run::SuccessfulRunRepo},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct MemorySuccessfulRunRepo {
    elements: Arc<RwLock<Vec<SuccessfulRun>>>,
}

#[async_trait]
impl SuccessfulRunRepo for MemorySuccessfulRunRepo {
    async fn get(
        &self,
        run_id: &crate::domain::run::id::RunId,
    ) -> crate::storage::error::Result<Option<SuccessfulRun>> {
        let job = self
            .elements
            .read()
            .await
            .iter()
            .find(|job| job.run_id() == *run_id)
            .cloned();
        Ok(job)
    }

    async fn add(&self, run: SuccessfulRun) -> crate::storage::error::Result<()> {
        let existing_job = self.get(&run.run_id()).await?;
        if existing_job.is_some() {
            return Err(Error::AlreadyExists);
        }

        self.elements.write().await.push(run);
        Ok(())
    }
}
