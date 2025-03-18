use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};

use super::id::JobId;

#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct RunningJob {
    id: JobId,
    created_at: DateTime<Utc>,
    started_at: DateTime<Utc>,
}

impl RunningJob {
    pub fn new(id: JobId, created_at: DateTime<Utc>, started_at: DateTime<Utc>) -> Self {
        Self {
            id,
            created_at,
            started_at,
        }
    }
}
