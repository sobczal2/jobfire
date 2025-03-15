use chrono::{Duration, Utc};
use tokio::{sync::mpsc, time::timeout};

use crate::{
    domain::job::{Context, FailedJob, PendingJob, RunningJob, SuccessfulJob},
    persistence::Persistence,
};

use super::error::{Error, Result};

pub(crate) enum JobWorkerCommand {
    Stop,
}

pub(crate) struct JobWorkerHandle {
    tx: mpsc::Sender<JobWorkerCommand>,
}

impl JobWorkerHandle {
    pub async fn stop(&self) -> Result<()> {
        self.tx
            .send(JobWorkerCommand::Stop)
            .await
            .map_err(|_| Error::StopFailed)
    }
}

pub(crate) struct JobWorkerSettings {
    poll_rate: Duration,
    command_channel_size: usize,
}

impl JobWorkerSettings {
    pub(crate) fn new(poll_rate: Duration, command_channel_size: usize) -> Result<Self> {
        if poll_rate < Duration::zero() {
            return Err(Error::InvalidSettings(
                "poll_rate can't be negative".to_owned(),
            ));
        }

        Ok(Self {
            poll_rate,
            command_channel_size,
        })
    }
}

pub(crate) struct JobWorker<T: Context> {
    settings: JobWorkerSettings,
    rx: mpsc::Receiver<JobWorkerCommand>,
    persistence: Persistence<T>,
    context: T,
    stop_requested: bool,
}

impl<T: Context> JobWorker<T> {
    pub(crate) fn start(
        settings: JobWorkerSettings,
        persistence: Persistence<T>,
        context: T,
    ) -> JobWorkerHandle {
        let (tx, rx) = mpsc::channel(settings.command_channel_size);
        let worker = Self {
            settings,
            rx,
            persistence,
            context,
            stop_requested: false,
        };

        tokio::spawn(async move { worker.run().await });

        JobWorkerHandle { tx }
    }

    fn stop(&mut self) {
        log::info!("stop requested");
        self.stop_requested = true;
    }

    async fn run(mut self) {
        loop {
            if self.stop_requested {
                log::info!("stopped");
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

            match self.persistence.pending_job_repo().find_scheduled().await {
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
    async fn handle_pending_job(&self, pending_job: PendingJob<T>) {
        if self
            .persistence
            .pending_job_repo()
            .delete(*pending_job.id())
            .await
            .is_err()
        {
            log::error!("failed to delete pending job");
            return;
        }

        let persistence = self.persistence.clone();
        let context = self.context.clone();

        tokio::spawn(async move {
            log::info!("running job: {}", pending_job.id());
            if persistence
                .running_job_repo()
                .add(RunningJob::new(
                    *pending_job.id(),
                    *pending_job.created_at(),
                    Utc::now(),
                ))
                .await
                .is_err()
            {
                log::error!("failed to add running job");
                if persistence
                    .pending_job_repo()
                    .add(pending_job.clone())
                    .await
                    .is_err()
                {
                    log::error!(
                        "failed to recover after issue, job: {} has been dropped",
                        pending_job.id()
                    )
                }
                return;
            }
            match pending_job.r#impl().run(context.clone()).await {
                Ok(report) => {
                    pending_job.r#impl().on_success(context).await;
                    if persistence
                        .successful_job_repo()
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
                Err(error) => {
                    pending_job.r#impl().on_fail(context).await;
                    if persistence
                        .failed_job_repo()
                        .add(FailedJob::new(
                            *pending_job.id(),
                            *pending_job.created_at(),
                            Utc::now(),
                            error,
                        ))
                        .await
                        .is_err()
                    {
                        log::error!("failed to save failed job report");
                    }
                }
            }
        });
    }
}
