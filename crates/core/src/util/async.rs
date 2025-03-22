use tokio::time::Interval;

pub async fn poll_predicate<P>(predicate: P, mut interval: Interval)
where
    P: AsyncFn() -> bool,
{
    loop {
        interval.tick().await;

        if predicate().await {
            return;
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::{Duration, Utc};
    use tokio::time::interval;

    use crate::util::r#async::poll_predicate;

    #[tokio::test]
    async fn predicate_runs_instantly() {
        // arrange
        let interval_duration = Duration::milliseconds(10);
        let interval = interval(interval_duration.to_std().unwrap());
        let now = Utc::now();

        // act
        poll_predicate(async || true, interval).await;

        // assert
        assert!(Utc::now() - now < interval_duration);
    }
}
