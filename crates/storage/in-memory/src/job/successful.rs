use std::sync::{Arc, RwLock};

use jobfire_core::{
    async_trait,
    prelude::{JobContext, SuccessfulJob},
    storage::{Error, SuccessfulJobRepo},
};

pub(crate) struct SuccessfulJobRepoImpl<T: JobContext> {
    elements: Arc<RwLock<Vec<SuccessfulJob<T>>>>,
}

impl<T: JobContext> Default for SuccessfulJobRepoImpl<T> {
    fn default() -> Self {
        Self {
            elements: Default::default(),
        }
    }
}

#[async_trait]
impl<T: JobContext> SuccessfulJobRepo<T> for SuccessfulJobRepoImpl<T> {
    async fn add(&self, successful_job: SuccessfulJob<T>) -> jobfire_core::storage::Result<()> {
        self.elements
            .write()
            .map_err(|e| Error::new(e.to_string()))?
            .push(successful_job);
        Ok(())
    }
}
