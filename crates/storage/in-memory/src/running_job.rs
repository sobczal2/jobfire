use std::sync::Arc;

use jobfire_core::{
    async_trait,
    domain::job::{id::JobId, running::RunningJob},
    storage::{
        error::{Error, Result},
        job::{AddJob, DeleteJob, GetJob, RunningJobRepo},
    },
};
use tokio::sync::RwLock;

use crate::{impl_add_job, impl_delete_job, impl_get_job};

#[derive(Default)]
pub(crate) struct RunningJobRepoImpl {
    elements: Arc<RwLock<Vec<RunningJob>>>,
}

impl_get_job!(RunningJobRepoImpl, RunningJob);
impl_add_job!(RunningJobRepoImpl, RunningJob);
impl_delete_job!(RunningJobRepoImpl, RunningJob);

#[async_trait]
impl RunningJobRepo for RunningJobRepoImpl {}
