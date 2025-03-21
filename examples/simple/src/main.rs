use chrono::{Duration, Utc};
use jobfire_core::{
    Uuid, async_trait,
    builders::Builder,
    domain::job::{
        context::{JobContext, JobContextData},
        error::Result,
        r#impl::{JobImpl, JobImplName},
        pending::PendingJob,
        report::Report,
        scheduler::JobScheduler,
    },
    managers::jobfire_manager::JobfireManager,
};
use jobfire_storage_in_memory::WithInMemoryStorage;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::{ops::AddAssign, sync::Mutex};
use tokio::{signal::ctrl_c, time::sleep};

struct SimpleContextData {
    counter: Mutex<usize>,
}

impl SimpleContextData {
    fn increment(&self) {
        self.counter.lock().unwrap().add_assign(1);
    }

    fn read(&self) -> usize {
        *self.counter.lock().unwrap()
    }
}

impl JobContextData for SimpleContextData {}

#[derive(Serialize, Deserialize)]
struct SimpleJobImpl {
    xd: Uuid,
}

#[async_trait]
impl JobImpl<SimpleContextData> for SimpleJobImpl {
    fn name() -> JobImplName {
        JobImplName::new("simple".to_owned())
    }

    async fn run(&self, context: JobContext<SimpleContextData>) -> Result<Report> {
        let context = context.data();
        context.increment();
        log::info!("Job number {} run", context.read());
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
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let context_data = SimpleContextData {
        counter: Mutex::new(0),
    };

    let manager = JobfireManager::builder(context_data)
        .with_in_memory_storage()
        .register_job_impl::<SimpleJobImpl>()
        .build()
        .unwrap();

    let now = Utc::now();

    let jobs = vec![
        PendingJob::new_at(now, SimpleJobImpl { xd: Uuid::now_v7() }).unwrap(),
        PendingJob::new_at(
            now - Duration::seconds(10)
                - Duration::microseconds(now.timestamp_subsec_micros() as i64),
            SimpleJobImpl { xd: Uuid::now_v7() },
        )
        .unwrap(),
        PendingJob::new_at(
            now + Duration::seconds(5)
                - Duration::microseconds(now.timestamp_subsec_micros() as i64),
            SimpleJobImpl { xd: Uuid::now_v7() },
        )
        .unwrap(),
        PendingJob::new_at(
            now + Duration::seconds(10)
                - Duration::microseconds(now.timestamp_subsec_micros() as i64),
            SimpleJobImpl { xd: Uuid::now_v7() },
        )
        .unwrap(),
        PendingJob::new_at(
            now + Duration::seconds(15)
                - Duration::microseconds(now.timestamp_subsec_micros() as i64),
            SimpleJobImpl { xd: Uuid::now_v7() },
        )
        .unwrap(),
    ];

    for job in jobs.iter() {
        manager.schedule(job).await.unwrap();
    }

    ctrl_c().await.unwrap();

    manager.stop().await.unwrap();
}
