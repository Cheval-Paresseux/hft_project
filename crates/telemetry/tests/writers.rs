mod common;

use common::{registry, TEST_METRIC_1, TEST_SCOPE_1};
use telemetry::tracing::capture::Scope;
use telemetry::tracing::handoff::{Channel, Subscriber};
use telemetry::tracing::writers::MemoryWriter;

// ── Test ──────────────────────────────────────────────────────────────────────

#[test]
fn records_are_dispatched_to_every_writer() {
    let registry = registry();
    let channel = Channel::unbounded();

    let writer_a = MemoryWriter::default();
    let writer_b = MemoryWriter::default();
    let records_a = writer_a.records.clone();
    let records_b = writer_b.records.clone();

    let mut subscriber = Subscriber::new(channel.receiver.clone(), &registry);
    subscriber.add_writer(writer_a);
    subscriber.add_writer(writer_b);

    {
        let mut scope = Scope::init(TEST_SCOPE_1, channel.sender, None);
        scope.metric(TEST_METRIC_1, 42, None);
    }

    subscriber.drain();

    assert_eq!(records_a.lock().unwrap().len(), 1);
    assert_eq!(records_b.lock().unwrap().len(), 1);
}

#[test]
fn subscriber_without_writers_drains_silently() {
    let registry = registry();
    let channel = Channel::unbounded();
    let mut subscriber = Subscriber::new(channel.receiver.clone(), &registry);

    {
        let mut scope = Scope::init(TEST_SCOPE_1, channel.sender, None);
        scope.metric(TEST_METRIC_1, 42, None);
    }

    subscriber.drain();
}
