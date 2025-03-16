use std::sync::{Arc, RwLock};

use jobfire_core::{
    async_trait,
    domain::{error::Error, job::SuccessfulJob},
    storage::job::SuccessfulJobRepo,
};

pub(crate) struct SuccessfulJobRepoImpl {
    elements: Arc<RwLock<Vec<SuccessfulJob>>>,
}

impl Default for SuccessfulJobRepoImpl {
    fn default() -> Self {
        Self {
            elements: Default::default(),
        }
    }
}

#[async_trait]
impl SuccessfulJobRepo for SuccessfulJobRepoImpl {
    async fn add(&self, successful_job: SuccessfulJob) -> jobfire_core::storage::error::Result<()> {
        self.elements
            .write()
            .map_err(|e| jobfire_core::storage::error::Error::new(e.to_string()))?
            .push(successful_job);
        Ok(())
    }
}
