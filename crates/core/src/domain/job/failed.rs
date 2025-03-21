use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};

use super::{error::Error, id::JobId};

#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct FailedJob {
    job_id: JobId,
    scheduled_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    error: Error,
}

impl FailedJob {
    pub fn new(
        job_id: JobId,
        scheduled_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        error: Error,
    ) -> Self {
        Self {
            job_id,
            scheduled_at,
            finished_at,
            error,
        }
    }
}
