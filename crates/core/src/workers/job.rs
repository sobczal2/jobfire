use chrono::{Duration, Utc};
use tokio::{
    sync::{mpsc, oneshot},
    time::{sleep, timeout},
};

use crate::{
    domain::{FailedJob, JobContext, PendingJob, RunningJob, SuccessfulJob},
    storage::Storage,
};

use super::error::{Error, Result};

pub(crate) enum JobWorkerCommand {
    Stop(oneshot::Sender<Result<()>>),
}

#[derive(Clone)]
pub(crate) struct JobWorkerHandle {
    tx: mpsc::Sender<JobWorkerCommand>,
}

impl JobWorkerHandle {
    pub async fn stop(&self) -> Result<()> {
        let (rx, tx) = oneshot::channel();
        self.tx
            .send(JobWorkerCommand::Stop(rx))
            .await
            .map_err(|_| Error::StopFailed)?;

        tx.await.unwrap()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct JobWorkerSettings {
    poll_rate: Duration,
    command_channel_size: usize,
}

impl JobWorkerSettings {
    pub fn new(poll_rate: Duration, command_channel_size: usize) -> Result<Self> {
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

impl Default for JobWorkerSettings {
    fn default() -> Self {
        Self::new(Duration::milliseconds(100), 32).unwrap()
    }
}

pub(crate) struct JobWorker<T: JobContext> {
    settings: JobWorkerSettings,
    rx: mpsc::Receiver<JobWorkerCommand>,
    storage: Storage<T>,
    context: T,
    stop_requested: bool,
    stopped: bool,
}

impl<T: JobContext> JobWorker<T> {
    pub(crate) fn start(
        settings: JobWorkerSettings,
        storage: Storage<T>,
        context: T,
    ) -> JobWorkerHandle {
        let (tx, rx) = mpsc::channel(settings.command_channel_size);
        let worker = Self {
            settings,
            rx,
            storage,
            context,
            stop_requested: false,
            stopped: false,
        };

        tokio::spawn(async move { worker.run().await });

        log::info!("JobWorker started");
        JobWorkerHandle { tx }
    }

    fn stop(&mut self) {
        log::info!("JobWorker stopping");
        self.stop_requested = true;
    }

    async fn run(mut self) {
        loop {
            if self.stop_requested {
                log::info!("JobWorker stopped");
                break;
            }

            if let Ok(Some(command)) =
                timeout(self.settings.poll_rate.to_std().unwrap(), self.rx.recv()).await
            {
                self.handle_command(command).await;
                continue;
            }

            match self.storage.pending_job_repo().find_scheduled().await {
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

    async fn handle_command(&mut self, command: JobWorkerCommand) {
        match command {
            JobWorkerCommand::Stop(sender) => {
                self.stop();
                // todo replace
                sleep(std::time::Duration::from_secs(3)).await;
                if sender.send(Ok(())).is_err() {
                    log::error!("failed to send back stop confirmation");
                }
            }
        }
    }
    async fn handle_pending_job(&self, pending_job: PendingJob<T>) {
        if self
            .storage
            .pending_job_repo()
            .delete(*pending_job.id())
            .await
            .is_err()
        {
            log::error!("failed to delete pending job");
            return;
        }

        let persistence = self.storage.clone();
        let context = self.context.clone();

        tokio::spawn(async move {
            log::info!("running job: {:?}", pending_job.id());
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
                        "failed to recover after issue, job: {:?} has been dropped",
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
