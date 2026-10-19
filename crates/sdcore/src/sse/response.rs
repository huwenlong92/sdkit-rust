use axum::response::sse::{Event, KeepAlive, Sse};
use std::convert::Infallible;

use super::SseReceiver;

pub fn receiver_to_sse(
    mut receiver: SseReceiver,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        yield Ok(Event::default().event("connected").data("{}"));
        while let Some(message) = receiver.recv().await {
            yield Ok(Event::default().event(message.event).data(message.data));
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// A session predicate is rechecked between events and at least once a second.
/// Returning false terminates long-lived authenticated streams on logout/expiry.
pub fn bounded_receiver_to_sse<F>(
    mut receiver: super::BoundedSseReceiver,
    authorized: F,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>
where
    F: Fn() -> bool + Send + 'static,
{
    let stream = async_stream::stream! {
        if authorized() { yield Ok(Event::default().event("connected").data("{}")); }
        let mut check = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            if !authorized() { break; }
            tokio::select! {
                message = receiver.recv() => {
                    let Some(message) = message else { break; };
                    if !authorized() { break; }
                    yield Ok(Event::default().event(message.event).data(message.data));
                }
                _ = check.tick() => {}
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}
