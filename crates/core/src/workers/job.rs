use chrono::{Duration, Utc};
use tokio::{
    sync::{mpsc, oneshot},
    time::{sleep, timeout},
};

use crate::{
    domain::job::JobContext,
    runners::job::{JobRunner, JobRunnerInput},
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

pub(crate) struct JobWorker<TJobContext: JobContext> {
    settings: JobWorkerSettings,
    rx: mpsc::Receiver<JobWorkerCommand>,
    storage: Storage,
    context: TJobContext,
    job_runner: Box<dyn JobRunner<TJobContext>>,
    stop_requested: bool,
    stopped: bool,
}

impl<TJobContext: JobContext> JobWorker<TJobContext> {
    pub(crate) fn start(
        settings: JobWorkerSettings,
        storage: Storage,
        context: TJobContext,
        job_runner: Box<dyn JobRunner<TJobContext>>,
    ) -> JobWorkerHandle {
        let (tx, rx) = mpsc::channel(settings.command_channel_size);
        let worker = Self {
            settings,
            rx,
            storage,
            context,
            job_runner,
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

            match self.storage.pending_job_repo().pop_scheduled().await {
                Ok(pending_job) => {
                    if let Some(pending_job) = pending_job {
                        let input = JobRunnerInput::new(pending_job);
                        self.job_runner.run(&input);
                    }
                    continue;
                }
                Err(err) => {
                    log::error!("failed to retrieve pending job: {err}");
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
                sleep(self.settings.poll_rate.to_std().unwrap()).await;
                if sender.send(Ok(())).is_err() {
                    log::error!("failed to send back stop confirmation");
                }
            }
        }
    }
}
