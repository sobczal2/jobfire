use super::id::JobId;
use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Clone, Getters, Serialize, Deserialize, Hash)]
#[getset(get = "pub")]
pub struct PendingJob {
    job_id: JobId,
    scheduled_at: DateTime<Utc>,
}

impl PendingJob {
    pub(crate) fn new(job_id: JobId, scheduled_at: DateTime<Utc>) -> Self {
        Self {
            job_id,
            scheduled_at,
        }
    }

    pub fn reschedule(&mut self, new_scheduled_at: DateTime<Utc>) {
        self.scheduled_at = new_scheduled_at;
    }
}
