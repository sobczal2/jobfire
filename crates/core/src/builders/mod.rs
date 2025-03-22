use thiserror::Error;

pub mod job_actions_registry;
pub mod job_manager;
pub mod job_scheduler;

#[derive(Clone, Error, Debug)]
pub enum Error {
    #[error("element missing: {element_name}")]
    ElementMissing { element_name: String },
    #[error("inner build failed: {0}")]
    InnerBuildFailed(Box<Error>),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Builder<T> {
    fn build(self) -> Result<T>;
}
