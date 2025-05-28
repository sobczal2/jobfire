use jobfire_core::{
    domain::job::{context::EmptyContextData, report::Report},
    managers::job_manager::JobManager,
    storage::memory::AddMemoryStorageService,
};
use jobfire_ephemeral::{AddEphemeralExtension, ScheduleEphemeralJob};
use simple_logger::SimpleLogger;
use tokio::{signal::ctrl_c, time::sleep};

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let manager = JobManager::new_default(EmptyContextData, |builder| {
        builder.add_ephemeral_extension::<EmptyContextData>();
        builder.add_memory_storage();
    })
    .unwrap();

    for i in 0..100 {
        let _job_id = manager
            .schedule_simple_ephemeral_job_now(move |_| async move {
                sleep(std::time::Duration::from_secs(1)).await;
                log::info!("hello from ephemeral job: {}", i);
                Ok(Report::new())
            })
            .await
            .unwrap();
    }

    ctrl_c().await.unwrap();

    manager.stop().await.unwrap();
}
