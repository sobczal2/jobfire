use std::sync::{Arc, Mutex};

use chrono::{Duration, Utc};
use jobfire_core::{
    Uuid, async_trait,
    domain::job::{
        context::{JobContext, JobContextData},
        error::{Error, Result},
        r#impl::{JobImpl, JobImplName},
        pending::PendingJob,
        report::Report,
    },
    managers::{
        JobfireManager,
        job_scheduler::{self, SimpleJobScheduler},
    },
    registries::job_actions::JobActionsRegistry,
    runners::simple::{
        job::SimpleJobRunner, on_fail::SimpleOnFailRunner, on_success::SimpleOnSuccessRunner,
    },
    storage::Storage,
    workers::job::JobWorkerSettings,
};
use jobfire_storage_in_memory::InMemoryStorage;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use tokio::{signal::ctrl_c, time::sleep};

#[derive(Clone)]
struct SimpleContextData {
    counter: Arc<Mutex<usize>>,
}

impl JobContextData for SimpleContextData {}

#[derive(Serialize, Deserialize)]
struct SimpleJob {
    xd: Uuid,
}

#[async_trait]
impl JobImpl<SimpleContextData> for SimpleJob {
    fn name() -> JobImplName {
        JobImplName::new("simple".to_owned())
    }

    async fn run(&self, context: JobContext<SimpleContextData>) -> Result<Report> {
        log::info!("Job run: {}", self.xd);
        sleep(std::time::Duration::from_secs_f32(11f32)).await;
        Ok(Report::new())
    }

    async fn on_success(&self, context: JobContext<SimpleContextData>) -> Result<()> {
        log::info!("on_sucess ran: {}", self.xd);
        Ok(())
    }
    async fn on_fail(&self, context: JobContext<SimpleContextData>) -> Result<()> {
        log::info!("on_fail ran: {}", self.xd);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let context_data = SimpleContextData {
        counter: Arc::new(Mutex::new(0)),
    };
    let storage = Storage::from(InMemoryStorage::default());
    let job_scheduler = SimpleJobScheduler::new(storage.clone());
    let context = JobContext::new(Arc::new(context_data), Arc::new(job_scheduler));
    let mut job_actions_registry = JobActionsRegistry::default();
    job_actions_registry.register::<SimpleJob>();
    let on_fail_runner = SimpleOnFailRunner::new(
        storage.clone(),
        context.clone(),
        job_actions_registry.clone(),
    );
    let on_success_runner = SimpleOnSuccessRunner::new(
        storage.clone(),
        context.clone(),
        job_actions_registry.clone(),
    );
    let job_runner = SimpleJobRunner::new(
        storage.clone(),
        context.clone(),
        job_actions_registry,
        Box::new(on_success_runner),
        Box::new(on_fail_runner),
    );
    let job_worker_settings = JobWorkerSettings::default();
    let manager = JobfireManager::start(
        context.clone(),
        storage.clone(),
        Arc::new(job_runner),
        job_worker_settings,
    )
    .unwrap();

    let jobs = vec![
        PendingJob::new_at(Utc::now(), SimpleJob { xd: Uuid::now_v7() }).unwrap(),
        PendingJob::new_at(
            Utc::now() - Duration::seconds(10),
            SimpleJob { xd: Uuid::now_v7() },
        )
        .unwrap(),
        PendingJob::new_at(
            Utc::now() + Duration::seconds(5),
            SimpleJob { xd: Uuid::now_v7() },
        )
        .unwrap(),
        PendingJob::new_at(
            Utc::now() + Duration::seconds(10),
            SimpleJob { xd: Uuid::now_v7() },
        )
        .unwrap(),
        PendingJob::new_at(
            Utc::now() + Duration::seconds(15),
            SimpleJob { xd: Uuid::now_v7() },
        )
        .unwrap(),
    ];

    for job in jobs.iter() {
        manager.schedule(job).await.unwrap();
    }

    ctrl_c().await.unwrap();

    manager.stop().await.unwrap();
}
