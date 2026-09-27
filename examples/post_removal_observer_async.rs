use moka::{future::Cache, notification::RemovalCause};
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug)]
struct RemovalTicket {
    key: Arc<String>,
    value: String,
    cause: RemovalCause,
}

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = mpsc::channel(64);
    let cache = Cache::builder()
        .max_capacity(10_000)
        .post_removal_observer(move |key, value, cause| {
            // The observer must not wait for capacity. The application decides
            // how to reconcile state if its bounded queue is full or closed.
            let _ = sender.try_send(RemovalTicket { key, value, cause });
        })
        .build();

    let key = "alice".to_owned();
    cache.insert(key.clone(), "first".to_owned()).await;
    cache.insert(key, "second".to_owned()).await;

    let ticket = receiver.recv().await.expect("replacement ticket");
    assert_eq!(ticket.key.as_str(), "alice");
    assert_eq!(ticket.value, "first");
    assert_eq!(ticket.cause, RemovalCause::Replaced);
}
