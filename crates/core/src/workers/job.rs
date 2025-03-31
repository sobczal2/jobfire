use chrono::{Duration, Utc};
use std::sync::Arc;
use thiserror::Error;
use tokio::{
    sync::{RwLock, mpsc, oneshot},
    time,
};

use crate::{
    domain::job::{context::ContextData, pending::PendingJob},
    runners::job::JobRunner,
    storage::{self, Storage},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("not stopped")]
    NotStopped,
    #[error("stopping failed")]
    StoppingFailed,
    #[error("already stopped")]
    AlreadyStopped,
    #[error("lock poisoned")]
    LockPoisoned,
    #[error("invalid settings: {0}")]
    InvalidSettings(String),
    #[error("storage error: {0}")]
    StorageError(#[from] storage::error::Error),
    #[error("channel closed")]
    ChannelClosed,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) enum JobWorkerCommand {
    Stop(oneshot::Sender<Result<()>>),
}

#[derive(Clone)]
pub(crate) struct JobWorkerHandle {
    tx: mpsc::Sender<JobWorkerCommand>,
    state: Arc<RwLock<State>>,
}

impl JobWorkerHandle {
    pub fn new(tx: mpsc::Sender<JobWorkerCommand>, state: Arc<RwLock<State>>) -> Self {
        Self { tx, state }
    }

    pub async fn stop(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(JobWorkerCommand::Stop(tx))
            .await
            .map_err(|_| Error::ChannelClosed)?;

        rx.await.map_err(|_| Error::ChannelClosed)?
    }

    pub async fn get_state(&self) -> State {
        self.state.read().await.clone()
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum State {
    Starting,
    Started,
    Stopping,
    Stopped,
}

pub(crate) struct JobWorker<TData: ContextData> {
    settings: JobWorkerSettings,
    storage: Storage,
    job_runner: JobRunner<TData>,
    state: Arc<RwLock<State>>,
}

impl<TData: ContextData> JobWorker<TData> {
    pub fn new(
        settings: JobWorkerSettings,
        storage: Storage,
        job_runner: JobRunner<TData>,
    ) -> Self {
        Self {
            settings,
            storage,
            job_runner,
            state: Arc::new(RwLock::new(State::Stopped)),
        }
    }

    pub fn start(self) -> JobWorkerHandle {
        let (tx, rx) = mpsc::channel(self.settings.command_channel_size);
        let handle = JobWorkerHandle::new(tx.clone(), self.state.clone());
        tokio::spawn(async move {
            self.run(rx).await.unwrap();
        });

        handle
    }

    async fn read_state(&self) -> State {
        let state = self.state.read().await;

        log::debug!("reading state: {:?}", state);
        state.clone()
    }

    async fn write_state(&self, new_state: State) {
        log::debug!("writing state: {:?}", new_state);
        let mut state = self.state.write().await;
        *state = new_state;
    }

    async fn get_next_command(
        &self,
        rx: &mut mpsc::Receiver<JobWorkerCommand>,
    ) -> Result<JobWorkerCommand> {
        match rx.recv().await {
            Some(command) => Ok(command),
            None => Err(Error::ChannelClosed),
        }
    }

    async fn handle_command(&self, command: JobWorkerCommand) -> Result<()> {
        log::trace!("handling command: {:?}", command);
        match command {
            JobWorkerCommand::Stop(sender) => sender
                .send(self.hadle_stop_command().await)
                .map_err(|_| Error::ChannelClosed),
        }
    }

    async fn hadle_stop_command(&self) -> Result<()> {
        if self.read_state().await != State::Started {
            return Err(Error::AlreadyStopped);
        }

        self.write_state(State::Stopping).await;

        Ok(())
    }

    async fn get_next_pending_job(&self) -> Result<PendingJob> {
        let mut interval = time::interval(self.settings.poll_rate.to_std().unwrap());
        loop {
            interval.tick().await;

            match self
                .storage
                .pending_job_repo()
                .pop_scheduled(Utc::now())
                .await?
            {
                Some(pending_job) => return Ok(pending_job),
                None => continue,
            }
        }
    }

    async fn handle_pending_job(&self, pending_job: PendingJob) {
        log::trace!("handling pending_job with id: {:?}", pending_job.job_id());
        let job_runner = self.job_runner.clone();
        tokio::spawn(async move {
            job_runner.run(pending_job).await;
        });
    }

    async fn run(&self, mut rx: mpsc::Receiver<JobWorkerCommand>) -> Result<()> {
        if self.read_state().await != State::Stopped {
            return Err(Error::NotStopped);
        }

        log::trace!("JobWorker starting");
        self.write_state(State::Starting).await;

        loop {
            log::debug!("run loop start");
            if self.read_state().await == State::Stopping {
                self.write_state(State::Stopped).await;
                log::trace!("JobWorker stopped");
                break;
            }

            log::trace!("JobWorker started");
            self.write_state(State::Started).await;

            tokio::select! {
                command = self.get_next_command(&mut rx) => {
                    match command {
                        Ok(command) => {
                            let result = self.handle_command(command).await;
                            if result.is_err() {
                                log::error!("error occurred: {:?}", result.err().unwrap());
                            }
                        }
                        Err(error) => log::error!("error ocurred: {:?}", error),
                    }
                }
                pending_job = self.get_next_pending_job() => {
                    match pending_job {
                        Ok(pending_job) => self.handle_pending_job(pending_job).await,
                        Err(error) => log::error!("error ocurred: {:?}", error),
                    }
                }
            }
        }

        Ok(())
    }
}
