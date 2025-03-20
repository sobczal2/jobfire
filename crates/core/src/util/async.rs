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
