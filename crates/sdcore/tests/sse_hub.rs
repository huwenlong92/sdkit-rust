use sdcore::sse::{SseHub, SseMessage};

#[test]
fn publish_only_sends_to_matching_topic() {
    let hub = SseHub::default();
    let mut deploy = hub.subscribe("deploy");
    let mut other = hub.subscribe("other");

    hub.publish("deploy", SseMessage::new("output", "started"));

    assert_eq!(deploy.try_recv().unwrap().data, "started");
    assert!(other.try_recv().is_err());
}
