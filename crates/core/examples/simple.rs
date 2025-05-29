use async_trait::async_trait;
use chrono::{Duration, Utc};
use jobfire_core::{
    domain::job::{
        Job,
        context::{Context, ContextData},
        error::{JobError, JobResult},
        r#impl::{JobImpl, JobImplName},
        report::Report,
    },
    managers::job_manager::JobManager,
    policies::{instant_retry::InstantRetryPolicy, timeout::TimeoutPolicy},
    registries::{job_actions::JobActionsRegistryBuilder, policies::PolicyRegistryBuilder},
    storage::memory::AddMemoryStorageService,
};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::sync::Mutex;
use tokio::signal::ctrl_c;
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

    async fn run(&self, _context: Context<SimpleContextData>) -> JobResult<Report> {
        log::info!("job number started");
        Err(JobError::Custom {
            message: "xd".to_owned(),
        })
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

    let manager = JobManager::new_default(context_data, |builder| {
        let mut job_actions_registry = JobActionsRegistryBuilder::default();
        job_actions_registry.register::<SimpleJobImpl>();
        builder.add_service(job_actions_registry.build());

        let mut policy_registry = PolicyRegistryBuilder::<SimpleContextData>::default();
        policy_registry.register(InstantRetryPolicy::default());
        policy_registry.register(TimeoutPolicy::new(Duration::seconds(1)));
        builder.add_service(policy_registry.build());

        builder.add_memory_storage();
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
        let job = Job::from_impl(
            job_impl,
            Utc::now(),
            vec![Box::new(InstantRetryPolicy::default())],
        )
        .unwrap();
        manager.schedule(job, at).await.unwrap();
    }

    ctrl_c().await.unwrap();

    manager.stop().await.unwrap();
}
