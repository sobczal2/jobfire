use super::error::Result;
use crate::domain::run::{failed::FailedRun, id::RunId, successful::SuccessfulRun};
use async_trait::async_trait;

/// Repository interface for managing `SuccessfulRun` entities.
///
/// This trait defines the core operations for storing and retrieving successful runs
/// from a persistent storage. It encapsulates the storage layer for completed runs
/// that have successfully finished their execution.
#[async_trait]
pub trait SuccessfulRunRepo: Send + Sync + 'static {
    /// Retrieves a successful run by its run_id.
    ///
    /// # Parameters
    ///
    /// * `run_id` - The run_id of the successful run to retrieve.
    ///
    /// # Returns
    ///
    /// * `Result<Option<SuccessfulRun>>` - Returns the successful run if found, None if not found,
    ///   or an error if the retrieval operation failed.
    async fn get(&self, run_id: &RunId) -> Result<Option<SuccessfulRun>>;

    /// Adds a successful run to the repository.
    ///
    /// # Parameters
    ///
    /// * `run` - The successful run to add to the repository.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns success if the successful run was added successfully,
    ///   or an error if the operation failed.
    ///
    /// # Important
    ///
    /// Implementation may fail if a successful run with the same run_id already exists
    /// in storage.
    async fn add(&self, run: SuccessfulRun) -> Result<()>;
}

/// Repository interface for managing `FailedRun` entities.
///
/// This trait defines operations for storing and retrieving failed runs
/// from a persistent storage. Failed runs represent executions that encountered
/// errors or otherwise did not complete successfully.
#[async_trait]
pub trait FailedRunRepo: Send + Sync + 'static {
    /// Retrieves a failed run by its run_id.
    ///
    /// # Parameters
    ///
    /// * `run_id` - The run_id of the failed run to retrieve.
    ///
    /// # Returns
    ///
    /// * `Result<Option<FailedRun>>` - Returns the failed run if found, None if not found,
    ///   or an error if the retrieval operation failed.
    async fn get(&self, run_id: &RunId) -> Result<Option<FailedRun>>;

    /// Adds a failed run to the repository.
    ///
    /// # Parameters
    ///
    /// * `run` - The failed run to add to the repository.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns success if the failed run was added successfully,
    ///   or an error if the operation failed.
    ///
    /// # Important
    ///
    /// Implementation may fail if a failed run with the same run_id already exists
    /// in storage.
    async fn add(&self, run: FailedRun) -> Result<()>;
}
