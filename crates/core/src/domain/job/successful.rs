use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};

use super::{id::JobId, report::Report};

#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct SuccessfulJob {
    job_id: JobId,
    scheduled_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    report: Report,
}

impl SuccessfulJob {
    pub fn new(
        job_id: JobId,
        scheduled_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        report: Report,
    ) -> Self {
        Self {
            job_id,
            scheduled_at,
            finished_at,
            report,
        }
    }
}
