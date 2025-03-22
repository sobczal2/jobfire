use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a job run.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct RunId(Uuid);

impl Default for RunId {
    fn default() -> Self {
        Self::new()
    }
}

impl RunId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}
