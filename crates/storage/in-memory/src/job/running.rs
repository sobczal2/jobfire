use std::sync::{Arc, RwLock};

use jobfire_core::{
    async_trait,
    domain::{error::Error, job::RunningJob},
    storage::job::RunningJobRepo,
};

pub(crate) struct RunningJobRepoImpl {
    elements: Arc<RwLock<Vec<RunningJob>>>,
}

impl Default for RunningJobRepoImpl {
    fn default() -> Self {
        Self {
            elements: Default::default(),
        }
    }
}

#[async_trait]
impl RunningJobRepo for RunningJobRepoImpl {
    async fn add(&self, running_job: RunningJob) -> jobfire_core::storage::error::Result<()> {
        self.elements
            .write()
            .map_err(|e| jobfire_core::storage::error::Error::new(e.to_string()))?
            .push(running_job);
        Ok(())
    }
}
