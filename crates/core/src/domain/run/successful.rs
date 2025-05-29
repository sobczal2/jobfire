use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::job::{id::JobId, report::Report};

use super::id::RunId;

/// Successful run information. run_id is unique
#[derive(Clone, Serialize, Deserialize)]
pub struct SuccessfulRun {
    run_id: RunId,
    job_id: JobId,
    scheduled_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    report: Report,
}

impl SuccessfulRun {
    pub fn new(
        run_id: RunId,
        job_id: JobId,
        scheduled_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        report: Report,
    ) -> Self {
        Self {
            run_id,
            job_id,
            scheduled_at,
            finished_at,
            report,
        }
    }

    pub fn run_id(&self) -> RunId {
        self.run_id
    }

    pub fn job_id(&self) -> JobId {
        self.job_id
    }

    pub fn scheduled_at(&self) -> DateTime<Utc> {
        self.scheduled_at
    }

    pub fn finished_at(&self) -> DateTime<Utc> {
        self.finished_at
    }

    pub fn report(&self) -> &Report {
        &self.report
    }
}
