use std::sync::{Arc, RwLock};

use jobfire_core::{
    async_trait,
    prelude::{JobContext, RunningJob},
    storage::{Error, RunningJobRepo},
};

pub(crate) struct RunningJobRepoImpl<T: JobContext> {
    elements: Arc<RwLock<Vec<RunningJob<T>>>>,
}

impl<T: JobContext> Default for RunningJobRepoImpl<T> {
    fn default() -> Self {
        Self {
            elements: Default::default(),
        }
    }
}

#[async_trait]
impl<T: JobContext> RunningJobRepo<T> for RunningJobRepoImpl<T> {
    async fn add(&self, running_job: RunningJob<T>) -> jobfire_core::storage::Result<()> {
        self.elements
            .write()
            .map_err(|e| Error::new(e.to_string()))?
            .push(running_job);
        Ok(())
    }
}
