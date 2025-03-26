use chrono::{Duration, Utc};
use jobfire_core::{
    Uuid, async_trait,
    domain::job::{
        context::{Context, ContextData},
        error::JobResult,
        r#impl::{JobImpl, JobImplName},
        report::Report,
    },
    managers::job_manager::JobManager,
    registries::builders::AddActionsRegistryService,
    storage::memory::AddMemoryStorageService,
};
use jobfire_ephemeral::{AddEphemeralExtension, RegisterEphemeralJob, ScheduleEphemeralJob};
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

impl ContextData for SimpleContextData {}

#[derive(Serialize, Deserialize)]
struct SimpleJobImpl {
    xd: Uuid,
}

#[async_trait]
impl JobImpl<SimpleContextData> for SimpleJobImpl {
    fn name() -> JobImplName {
        JobImplName::new("simple".to_owned())
    }

    async fn run(&self, context: Context<SimpleContextData>) -> JobResult<Report> {
        let context = context.data();
        context.increment();
        log::info!("Job number {} run", context.read());
        sleep(std::time::Duration::from_secs_f32(11f32)).await;
        Ok(Report::new())
    }

    async fn on_success(&self, _context: Context<SimpleContextData>) {
        log::info!("on_sucess ran: {}", self.xd);
    }
    async fn on_fail(&self, _context: Context<SimpleContextData>) {
        log::info!("on_fail ran: {}", self.xd);
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

    let manager = JobManager::new_default(context_data, |builder| {
        builder.add_job_actions_registry(|jr_builder| {
            jr_builder.register::<SimpleJobImpl>();
            jr_builder.register_ephemeral_job();
        });
        builder.add_memory_storage();
        builder.add_ephemeral_extension();
    })
    .unwrap();

    for _ in 0..100 {
        let _job_id = manager
            .schedule_simple_ephemeral_job_now(async |_| {
                sleep(std::time::Duration::from_secs(11)).await;
                log::info!("hello from ephemeral job");
                Ok(Report::new())
            })
            .await
            .unwrap();
    }

    let now = Utc::now();

    let jobs = vec![
        (
            SimpleJobImpl { xd: Uuid::now_v7() },
            now - Duration::seconds(10),
        ),
        (
            SimpleJobImpl { xd: Uuid::now_v7() },
            now + Duration::seconds(5),
        ),
        (
            SimpleJobImpl { xd: Uuid::now_v7() },
            now + Duration::seconds(10),
        ),
        (
            SimpleJobImpl { xd: Uuid::now_v7() },
            now + Duration::seconds(15),
        ),
        (
            SimpleJobImpl { xd: Uuid::now_v7() },
            now + Duration::seconds(20),
        ),
        (
            SimpleJobImpl { xd: Uuid::now_v7() },
            now + Duration::seconds(25),
        ),
    ];

    for (job_impl, at) in jobs.into_iter() {
        manager.schedule(job_impl, at).await.unwrap();
    }

    ctrl_c().await.unwrap();

    manager.stop().await.unwrap();
}
