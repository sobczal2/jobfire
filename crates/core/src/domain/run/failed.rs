use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::job::{error::JobError, id::JobId};

use super::id::RunId;

/// Failed run information. run_id is unique
#[derive(Clone, Serialize, Deserialize)]
pub struct FailedRun {
    run_id: RunId,
    job_id: JobId,
    scheduled_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    error: JobError,
}

impl FailedRun {
    pub fn new(
        run_id: RunId,
        job_id: JobId,
        scheduled_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        error: JobError,
    ) -> Self {
        Self {
            run_id,
            job_id,
            scheduled_at,
            finished_at,
            error,
        }
    }

    pub fn job_id(&self) -> JobId {
        self.job_id
    }

    pub fn run_id(&self) -> RunId {
        self.run_id
    }
}
