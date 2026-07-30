use telemetry::schema::{metrics, scopes};
use telemetry::tracing::capture::{MetricId, ScopeId};
use telemetry::tracing::enrichment::{MetricMetadata, ScopeMetadata};
use telemetry::tracing::{
    capture::Scope,
    enrichment::Registry,
    handoff::{Channel, Subscriber},
    writers::MemoryWriter,
};

// ── Metadata for Registry ─────────────────────────────────────────────────────

pub const TEST_SCOPE_1: ScopeId = ScopeId::new(0x100, 0);
pub const TEST_SCOPE_2: ScopeId = ScopeId::new(0x100, 1);

pub static TEST_SCOPES: &[ScopeMetadata] = &[
    ScopeMetadata {
        id: TEST_SCOPE_1,
        name: "test_scope_1",
        description: "none",
        payload_info: "N/A",
    },
    ScopeMetadata {
        id: TEST_SCOPE_2,
        name: "test_scope_2",
        description: "none",
        payload_info: "N/A",
    },
];

pub const TEST_METRIC_1: MetricId = MetricId::new(0x100, 0);
pub const TEST_METRIC_2: MetricId = MetricId::new(0x100, 1);

pub static TEST_METRICS: &[MetricMetadata] = &[
    MetricMetadata {
        id: TEST_METRIC_1,
        name: "test_metric_1",
        unit: "none",
        description: "none",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: TEST_METRIC_2,
        name: "test_metric_2",
        unit: "none",
        description: "none",
        payload_info: "N/A",
    },
];

#[test]
fn integration_test() {
    let mut registry = Registry::default();
    registry.build_scopes(TEST_SCOPES);
    registry.build_metrics(TEST_METRICS);

    let writer = MemoryWriter::default();
    let records = writer.records.clone();

    let channel = Channel::unbounded();
    let mut subscriber = Subscriber::new(channel.receiver, &registry);
    subscriber.add_writer(writer);

    {
        let mut root_scope = Scope::init(scopes::INITIALIZATION, channel.sender, None);
        root_scope.process_memory_snapshot(); // telemetry standard metrics for memory
        root_scope.metric(TEST_METRIC_1, 42, None); // custom metric

        {
            let mut child_scope_1 = root_scope.child(TEST_SCOPE_1, None);
            child_scope_1.metric(TEST_METRIC_2, 102, Some(400));
        }

        {
            let mut child_scope_2 = root_scope.child(TEST_SCOPE_2, None);
            child_scope_2.metric(TEST_METRIC_2, 93, None);
        }
    }

    subscriber.drain();

    let records = records.lock().unwrap();

    assert_eq!(records.len(), 3);
}
