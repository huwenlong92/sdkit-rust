use sdcore::sse::{BoundedSseHub, SseMessage};
#[tokio::test]
async fn slow_subscribers_get_gaps_and_close_drains_then_ends() {
    let hub = BoundedSseHub::new(2);
    let mut a = hub.subscribe("a");
    for i in 0..10 {
        hub.publish("a", SseMessage::new("log", i.to_string()));
    }
    assert_eq!(a.recv().await.unwrap().event, "gap");
    assert_eq!(a.recv().await.unwrap().data, "8");
    hub.close();
    assert_eq!(a.recv().await.unwrap().data, "9");
    assert!(a.recv().await.is_none());
}

#[tokio::test]
async fn authenticated_stream_closes_when_session_is_revoked_even_without_events() {
    use sdcore::axum::{body::to_bytes, response::IntoResponse};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    let hub = BoundedSseHub::new(2);
    let valid = Arc::new(AtomicBool::new(true));
    let predicate = valid.clone();
    let body = sdcore::sse::bounded_receiver_to_sse(hub.subscribe("private"), move || {
        predicate.load(Ordering::SeqCst)
    })
    .into_response()
    .into_body();
    let collect = tokio::spawn(async move { to_bytes(body, 1024).await.unwrap() });
    tokio::task::yield_now().await;
    valid.store(false, Ordering::SeqCst);
    tokio::time::timeout(std::time::Duration::from_secs(2), collect)
        .await
        .unwrap()
        .unwrap();
}
