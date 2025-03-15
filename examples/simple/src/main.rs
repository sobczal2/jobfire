use std::sync::{Arc, Mutex};

use chrono::{Duration, Utc};
use jobfire_core::{prelude::*, workers::job::JobWorkerSettings};
use jobfire_storage_in_memory::InMemoryStorage;
use simple_logger::SimpleLogger;
use tokio::signal::ctrl_c;

#[derive(Clone)]
struct SimpleContext {
    counter: Arc<Mutex<usize>>,
}

impl JobContext for SimpleContext {}

struct SimpleJob;

#[async_trait]
impl JobImpl<SimpleContext> for SimpleJob {
    async fn run(&self, context: SimpleContext) -> Result<Report, Error> {
        let mut counter_lock = context.counter.lock().unwrap();
        log::info!("simple job ran with counter = {}", counter_lock);
        *counter_lock += 1;
        Ok(Report::new())
    }

    async fn on_fail(&self, context: SimpleContext) {
        log::info!("on fail ran")
    }

    async fn on_success(&self, context: SimpleContext) {
        log::info!("on success ran")
    }
}
#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let context = SimpleContext {
        counter: Arc::new(Mutex::new(0)),
    };
    let storage = InMemoryStorage::default();
    let manager =
        JobfireManager::start(context, storage.into(), JobWorkerSettings::default()).unwrap();

    let jobs = vec![
        PendingJob::new_at(Utc::now(), Arc::new(SimpleJob)),
        PendingJob::new_at(Utc::now() - Duration::seconds(10), Arc::new(SimpleJob)),
        PendingJob::new_at(Utc::now() + Duration::seconds(5), Arc::new(SimpleJob)),
        PendingJob::new_at(Utc::now() + Duration::seconds(10), Arc::new(SimpleJob)),
        PendingJob::new_at(Utc::now() + Duration::seconds(15), Arc::new(SimpleJob)),
    ];

    for job in jobs.iter() {
        manager.schedule(job.clone()).await.unwrap();
    }

    ctrl_c().await.unwrap();

    manager.stop().await.unwrap();
}
