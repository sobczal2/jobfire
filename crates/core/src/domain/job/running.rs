use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};

use super::id::JobId;

#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct RunningJob {
    job_id: JobId,
    started_at: DateTime<Utc>,
}

impl RunningJob {
    pub fn new(job_id: JobId, started_at: DateTime<Utc>) -> Self {
        Self { job_id, started_at }
    }
}
