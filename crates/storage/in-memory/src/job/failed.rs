use std::sync::{Arc, RwLock};

use jobfire_core::{
    async_trait,
    prelude::{FailedJob, JobContext},
    storage::{Error, FailedJobRepo},
};

pub(crate) struct FailedJobRepoImpl<T: JobContext> {
    elements: Arc<RwLock<Vec<FailedJob<T>>>>,
}

impl<T: JobContext> Default for FailedJobRepoImpl<T> {
    fn default() -> Self {
        Self {
            elements: Default::default(),
        }
    }
}

#[async_trait]
impl<T: JobContext> FailedJobRepo<T> for FailedJobRepoImpl<T> {
    async fn add(&self, failed_job: FailedJob<T>) -> jobfire_core::storage::Result<()> {
        self.elements
            .write()
            .map_err(|e| Error::new(e.to_string()))?
            .push(failed_job);
        Ok(())
    }
}
