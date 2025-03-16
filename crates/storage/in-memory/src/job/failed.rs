use std::sync::{Arc, RwLock};

use jobfire_core::{
    async_trait,
    domain::{error::Error, job::FailedJob},
    storage::job::FailedJobRepo,
};

pub(crate) struct FailedJobRepoImpl {
    elements: Arc<RwLock<Vec<FailedJob>>>,
}

impl Default for FailedJobRepoImpl {
    fn default() -> Self {
        Self {
            elements: Default::default(),
        }
    }
}

#[async_trait]
impl FailedJobRepo for FailedJobRepoImpl {
    async fn add(&self, failed_job: FailedJob) -> jobfire_core::storage::error::Result<()> {
        self.elements
            .write()
            .map_err(|e| jobfire_core::storage::error::Error::new(e.to_string()))?
            .push(failed_job);
        Ok(())
    }
}
