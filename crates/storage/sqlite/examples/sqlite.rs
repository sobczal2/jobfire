use async_trait::async_trait;
use chrono::{Duration, Utc};
use jobfire_core::{
    domain::job::{
        context::{Context, ContextData},
        error::JobResult,
        r#impl::{JobImpl, JobImplName},
        report::Report,
    },
    managers::job_manager::JobManager,
    registries::builders::AddActionsRegistryService,
    storage::AddStorageService,
};
use jobfire_storage_sqlite::SqliteStorage;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::sync::Mutex;
use tokio::{signal::ctrl_c, time::sleep};
use uuid::Uuid;

struct SimpleContextData {
    counter: Mutex<usize>,
}

impl SimpleContextData {
    fn increment(&self) {
        *self.counter.lock().unwrap() += 1;
    }

    fn read(&self) -> usize {
        *self.counter.lock().unwrap()
    }
}

impl ContextData for SimpleContextData {}

#[derive(Serialize, Deserialize)]
struct SimpleJobImpl {
    id: Uuid,
}

#[async_trait]
impl JobImpl<SimpleContextData> for SimpleJobImpl {
    fn name() -> JobImplName {
        JobImplName::new("simple".to_owned())
    }

    async fn run(&self, context: Context<SimpleContextData>) -> JobResult<Report> {
        let context = context.data();
        context.increment();
        log::info!("job number {} run", context.read());
        sleep(std::time::Duration::from_secs_f32(11f32)).await;
        Ok(Report::new())
    }

    async fn on_success(&self, _context: Context<SimpleContextData>) {
        log::info!("on_sucess ran: {}", self.id);
    }
    async fn on_fail(&self, _context: Context<SimpleContextData>) {
        log::info!("on_fail ran: {}", self.id);
    }
}

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let context_data = SimpleContextData {
        counter: Mutex::new(0),
    };

    let storage = SqliteStorage::new_in_memory().await;

    let manager = JobManager::new_default(context_data, |builder| {
        builder.add_job_actions_registry(|jr_builder| {
            jr_builder.register::<SimpleJobImpl>();
        });
        builder.add_storage(storage);
    })
    .unwrap();

    let now = Utc::now();

    let jobs = vec![
        (
            SimpleJobImpl { id: Uuid::now_v7() },
            now - Duration::seconds(10),
        ),
        (
            SimpleJobImpl { id: Uuid::now_v7() },
            now + Duration::seconds(5),
        ),
        (
            SimpleJobImpl { id: Uuid::now_v7() },
            now + Duration::seconds(10),
        ),
        (
            SimpleJobImpl { id: Uuid::now_v7() },
            now + Duration::seconds(15),
        ),
        (
            SimpleJobImpl { id: Uuid::now_v7() },
            now + Duration::seconds(20),
        ),
        (
            SimpleJobImpl { id: Uuid::now_v7() },
            now + Duration::seconds(25),
        ),
    ];

    for (job_impl, at) in jobs.into_iter() {
        manager.schedule(job_impl, at).await.unwrap();
    }

    ctrl_c().await.unwrap();

    manager.stop().await.unwrap();
}
