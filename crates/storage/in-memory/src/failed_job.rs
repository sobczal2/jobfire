use std::sync::Arc;

use crate::{impl_add_job, impl_delete_job};

use jobfire_core::{
    async_trait,
    domain::job::{failed::FailedJob, id::JobId},
    storage::{
        error::{Error, Result},
        job::{AddJob, DeleteJob, FailedJobRepo},
    },
};
use tokio::sync::RwLock;

#[derive(Default)]
pub(crate) struct FailedJobRepoImpl {
    elements: Arc<RwLock<Vec<FailedJob>>>,
}

impl_add_job!(FailedJobRepoImpl, FailedJob);
impl_delete_job!(FailedJobRepoImpl, FailedJob);

#[async_trait]
impl FailedJobRepo for FailedJobRepoImpl {}
