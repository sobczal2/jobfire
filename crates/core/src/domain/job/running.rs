use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::run::id::RunId;

use super::id::JobId;

/// Job that is currently running.
///
/// This structure represents a job that is being executed. It tracks both
/// the job's identity and the specific execution instance (run).
#[derive(Clone, Serialize, Deserialize)]
pub struct RunningJob {
    /// Reference to the original job's identifier.
    ///
    /// This links back to the original `Job` from which this running
    /// instance was created.
    job_id: JobId,

    /// Unique identifier for this specific execution of the job.
    ///
    /// Both `run_id` and `job_id` are unique. The `run_id` identifies
    /// a specific execution, while `job_id` identifies the job definition.
    run_id: RunId,

    /// Timestamp when the job execution started.
    ///
    /// Used for tracking execution time and potentially for timeout management.
    started_at: DateTime<Utc>,
}

impl RunningJob {
    pub fn new(job_id: JobId, run_id: RunId, started_at: DateTime<Utc>) -> Self {
        Self {
            run_id,
            job_id,
            started_at,
        }
    }

    pub fn job_id(&self) -> JobId {
        self.job_id
    }

    pub fn run_id(&self) -> RunId {
        self.run_id
    }
}
