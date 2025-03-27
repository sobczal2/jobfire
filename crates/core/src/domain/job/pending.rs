use super::id::JobId;
use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};

/// Job that is scheduled for execution but not yet running.
///
/// This structure represents a job that has been scheduled for future
/// execution. It contains the minimum information needed to identify
/// and schedule the job.
#[derive(Clone, Getters, Serialize, Deserialize, Hash)]
#[getset(get = "pub")]
pub struct PendingJob {
    /// Reference to the original job's identifier.
    ///
    /// This links back to the original `Job` for which execution
    /// has been scheduled. Unique.
    job_id: JobId,

    /// Timestamp when the job is scheduled to be executed.
    ///
    /// The scheduler component uses this to determine which jobs
    /// are ready for execution.
    scheduled_at: DateTime<Utc>,
}

impl PendingJob {
    pub fn new(job_id: JobId, scheduled_at: DateTime<Utc>) -> Self {
        Self {
            job_id,
            scheduled_at,
        }
    }

    pub fn reschedule(&mut self, new_scheduled_at: DateTime<Utc>) {
        self.scheduled_at = new_scheduled_at;
    }
}
