use jobfire_core::{
    domain::job::{context::EmptyContextData, report::Report},
    managers::job_manager::JobManager,
    registries::builders::AddActionsRegistryService,
    storage::memory::AddMemoryStorageService,
};
use jobfire_ephemeral::{AddEphemeralExtension, RegisterEphemeralJob, ScheduleEphemeralJob};
use simple_logger::SimpleLogger;
use tokio::{signal::ctrl_c, time::sleep};

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let manager = JobManager::new_default(EmptyContextData, |builder| {
        builder.add_job_actions_registry(|jr_builder| {
            jr_builder.register_ephemeral_job();
        });
        builder.add_ephemeral_extension();
        builder.add_memory_storage();
    })
    .unwrap();

    for _ in 0..100 {
        let _job_id = manager
            .schedule_simple_ephemeral_job_now(async |_| {
                sleep(std::time::Duration::from_secs(1)).await;
                log::info!("hello from ephemeral job");
                Ok(Report::new())
            })
            .await
            .unwrap();
    }

    ctrl_c().await.unwrap();

    manager.stop().await.unwrap();
}
