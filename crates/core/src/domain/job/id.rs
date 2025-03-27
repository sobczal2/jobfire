use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a job.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct JobId(Uuid);

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn value(&self) -> &Uuid {
        &self.0
    }
}
