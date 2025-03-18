use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};

use super::{id::JobId, report::Report};

#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct SuccessfulJob {
    id: JobId,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    report: Report,
}

impl SuccessfulJob {
    pub fn new(
        id: JobId,
        created_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        report: Report,
    ) -> Self {
        Self {
            id,
            created_at,
            finished_at,
            report,
        }
    }
}
