use std::sync::Arc;

use jobfire_core::{
    async_trait,
    domain::job::{id::JobId, successful::SuccessfulJob},
    storage::{
        error::{Error, Result},
        job::{AddJob, DeleteJob, GetJob, SuccessfulJobRepo},
    },
};
use tokio::sync::RwLock;

use crate::{impl_add_job, impl_delete_job, impl_get_job};

#[derive(Default)]
pub(crate) struct SuccessfulJobRepoImpl {
    elements: Arc<RwLock<Vec<SuccessfulJob>>>,
}

impl_get_job!(SuccessfulJobRepoImpl, SuccessfulJob);
impl_add_job!(SuccessfulJobRepoImpl, SuccessfulJob);
impl_delete_job!(SuccessfulJobRepoImpl, SuccessfulJob);

#[async_trait]
impl SuccessfulJobRepo for SuccessfulJobRepoImpl {}
