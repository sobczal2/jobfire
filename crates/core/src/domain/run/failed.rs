use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::job::{Error, id::JobId};

use super::id::RunId;

/// Failed run information. run_id is unique
#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct FailedRun {
    run_id: RunId,
    job_id: JobId,
    scheduled_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    error: Error,
}

impl FailedRun {
    pub fn new(
        run_id: RunId,
        job_id: JobId,
        scheduled_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        error: Error,
    ) -> Self {
        Self {
            run_id,
            job_id,
            scheduled_at,
            finished_at,
            error,
        }
    }
}
