use std::sync::Arc;

use chrono::{Duration, Utc};
use tokio::{sync::mpsc, time::timeout};

use crate::{
    domain::job::{PendingJob, RunningJob, SuccessfulJob},
    persistence::job::{FailedJobRepo, PendingJobRepo, RunningJobRepo, SuccessfullJobRepo},
};

pub(crate) enum JobWorkerCommand {
    Stop,
}

pub(crate) struct JobWorkerHandle {
    tx: mpsc::Sender<JobWorkerCommand>,
}

pub(crate) struct JobWorkerSettings {
    poll_rate: Duration,
    command_channel_size: usize,
}

pub(crate) struct JobWorker {
    settings: JobWorkerSettings,
    rx: mpsc::Receiver<JobWorkerCommand>,
    pending_job_repo: Arc<dyn PendingJobRepo>,
    running_job_repo: Arc<dyn RunningJobRepo>,
    failed_job_repo: Arc<dyn FailedJobRepo>,
    successful_job_repo: Arc<dyn SuccessfullJobRepo>,
    stop_requested: bool,
}

impl JobWorker {
    pub(crate) fn start(
        settings: JobWorkerSettings,
        pending_job_repo: Arc<dyn PendingJobRepo>,
        running_job_repo: Arc<dyn RunningJobRepo>,
        failed_job_repo: Arc<dyn FailedJobRepo>,
        successful_job_repo: Arc<dyn SuccessfullJobRepo>,
    ) -> JobWorkerHandle {
        assert!(settings.poll_rate > Duration::zero());

        let (tx, rx) = mpsc::channel(settings.command_channel_size);
        let worker = Self {
            settings,
            rx,
            pending_job_repo,
            running_job_repo,
            failed_job_repo,
            successful_job_repo,
            stop_requested: false,
        };

        tokio::spawn(async move { worker.run().await });

        JobWorkerHandle { tx }
    }

    fn stop(&mut self) {
        self.stop_requested = true;
    }

    async fn run(mut self) {
        loop {
            if self.stop_requested {
                break;
            }

            if let Ok(command) =
                timeout(self.settings.poll_rate.to_std().unwrap(), self.rx.recv()).await
            {
                if let Some(command) = command {
                    self.handle_command(command);
                    continue;
                } else {
                    self.stop();
                }
            }

            match self.pending_job_repo.first_scheduled().await {
                Ok(mut pending_job) => {
                    while let Some(pending_job) = pending_job.take() {
                        self.handle_pending_job(pending_job).await
                    }
                    continue;
                }
                Err(_) => {
                    log::error!("failed to retrieve pending job");
                    continue;
                }
            }
        }
    }

    fn handle_command(&self, command: JobWorkerCommand) {}
    async fn handle_pending_job(&self, pending_job: PendingJob) {
        if self
            .pending_job_repo
            .delete(*pending_job.id())
            .await
            .is_err()
        {
            log::error!("failed to delete pending job");
            return;
        }

        let pending_job_repo = self.pending_job_repo.clone();
        let running_job_repo = self.running_job_repo.clone();
        let successful_job_repo = self.successful_job_repo.clone();
        let failed_job_repo = self.failed_job_repo.clone();

        tokio::spawn(async move {
            log::info!("running job: {}", pending_job.id());
            if running_job_repo
                .add(RunningJob::new(
                    *pending_job.id(),
                    *pending_job.created_at(),
                    Utc::now(),
                ))
                .await
                .is_err()
            {
                log::error!("failed to add running job");
                if pending_job_repo.add(pending_job.clone()).await.is_err() {
                    log::error!(
                        "failed to recover after issue, job: {} has been dropped",
                        pending_job.id()
                    )
                }
                return;
            }
            match pending_job.r#impl().run().await {
                Ok(report) => {
                    if successful_job_repo
                        .add(SuccessfulJob::new(
                            *pending_job.id(),
                            *pending_job.created_at(),
                            Utc::now(),
                            report,
                        ))
                        .await
                        .is_err()
                    {
                        log::error!("failed to save successful job report");
                    }
                }
                Err(_) => todo!(),
            }
        });
    }
}
