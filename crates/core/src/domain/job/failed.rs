use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};

use super::{error::Error, id::JobId};

#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct FailedJob {
    id: JobId,
    created_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    error: Error,
}

impl FailedJob {
    pub fn new(
        id: JobId,
        created_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        error: Error,
    ) -> Self {
        Self {
            id,
            created_at,
            finished_at,
            error,
        }
    }
}
