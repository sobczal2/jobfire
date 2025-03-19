use std::sync::{Arc, Mutex};

use chrono::{Duration, Utc};
use jobfire_core::{
    async_trait,
    domain::job::{
        context::{JobContext, JobContextData},
        error::{Error, Result},
        r#impl::{JobImpl, JobImplName},
        report::Report,
    },
    managers::JobfireManager,
    registries::job_actions::JobActionsRegistry,
    runners::simple::job::SimpleJobRunner,
};
use jobfire_storage_in_memory::InMemoryStorage;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use tokio::signal::ctrl_c;

#[derive(Clone)]
struct SimpleContextData {
    counter: Arc<Mutex<usize>>,
}

impl JobContextData for SimpleContextData {}

#[derive(Serialize, Deserialize)]
struct SimpleJob;

#[async_trait]
impl JobImpl<SimpleContextData> for SimpleJob {
    fn name() -> JobImplName {
        JobImplName::new("simple".to_owned())
    }

    async fn run(&self, context: JobContext<SimpleContextData>) -> Result<Report> {
        todo!()
    }

    async fn on_fail(&self, context: JobContext<SimpleContextData>) -> Result<()> {
        todo!()
    }

    async fn on_success(&self, context: JobContext<SimpleContextData>) -> Result<()> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let context = SimpleContextData {
        counter: Arc::new(Mutex::new(0)),
    };
    let storage = InMemoryStorage::default();
    let mut job_actions_registry = JobActionsRegistry::default();
    job_actions_registry.register::<SimpleJob>();
    let on_fail_runner = SimpleOnF
    let job_runner = SimpleJobRunner::new(
        storage,
        context,
        job_actions_registry,
        on_fail_runner,
        on_success_runner,
    );
    let manager = JobfireManager::start(context, storage).unwrap();

    let jobs = vec![
        PendingJob::new_at(Utc::now(), SimpleJob).unwrap(),
        PendingJob::new_at(Utc::now() - Duration::seconds(10), SimpleJob).unwrap(),
        PendingJob::new_at(Utc::now() + Duration::seconds(5), SimpleJob).unwrap(),
        PendingJob::new_at(Utc::now() + Duration::seconds(10), SimpleJob).unwrap(),
        PendingJob::new_at(Utc::now() + Duration::seconds(15), SimpleJob).unwrap(),
    ];

    for job in jobs.iter() {
        manager.schedule(job.clone()).await.unwrap();
    }

    ctrl_c().await.unwrap();

    manager.stop().await.unwrap();
}
