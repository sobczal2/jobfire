#[macro_export]
macro_rules! impl_get_job {
    ($job_repo_type:ty, $job_type:ty) => {
        #[async_trait]
        impl GetJob<$job_type> for $job_repo_type {
            async fn get(&self, job_id: &JobId) -> Result<Option<$job_type>> {
                Ok(self
                    .elements
                    .read()
                    .await
                    .iter()
                    .find(|job| job.job_id() == job_id)
                    .cloned())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_add_job {
    ($job_repo_type:ty, $job_type:ty) => {
        #[async_trait]
        impl AddJob<$job_type> for $job_repo_type {
            async fn add(&self, job: $job_type) -> Result<()> {
                self.elements.write().await.push(job);
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_delete_job {
    ($job_repo_type:ty, $job_type:ty) => {
        #[async_trait]
        impl DeleteJob<$job_type> for $job_repo_type {
            async fn delete(&self, job_id: &JobId) -> Result<$job_type> {
                let existing_index = self
                    .elements
                    .read()
                    .await
                    .iter()
                    .enumerate()
                    .find(|(_, e)| e.job_id() == job_id)
                    .map(|(i, _)| i);

                if existing_index.is_none() {
                    return Err(Error::NotFound);
                }
                let existing_index = existing_index.unwrap();

                let existing_element = self.elements.write().await.swap_remove(existing_index);

                Ok(existing_element)
            }
        }
    };
}
