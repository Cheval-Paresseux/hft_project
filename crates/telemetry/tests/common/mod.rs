#![allow(dead_code)]

use telemetry::tracing::capture::{MetricId, ScopeId};
use telemetry::tracing::enrichment::{
    MetricMetadata, Registry, ResolvedMetric, ResolvedRecord, ScopeMetadata,
};
use telemetry::tracing::handoff::{Channel, Subscriber};
use telemetry::tracing::writers::MemoryWriter;

use std::sync::{Arc, Mutex};

// ── Test Metadata ─────────────────────────────────────────────────────────────

pub const TEST_SCOPE_1: ScopeId = ScopeId::new(0x100, 0);
pub const TEST_SCOPE_2: ScopeId = ScopeId::new(0x100, 1);

pub static TEST_SCOPES: &[ScopeMetadata] = &[
    ScopeMetadata {
        id: TEST_SCOPE_1,
        name: "test_scope_1",
        description: "First test scope.",
        payload_info: "N/A",
    },
    ScopeMetadata {
        id: TEST_SCOPE_2,
        name: "test_scope_2",
        description: "Second test scope.",
        payload_info: "N/A",
    },
];

pub const TEST_METRIC_1: MetricId = MetricId::new(0x100, 0);
pub const TEST_METRIC_2: MetricId = MetricId::new(0x100, 1);

pub static TEST_METRICS: &[MetricMetadata] = &[
    MetricMetadata {
        id: TEST_METRIC_1,
        name: "test_metric_1",
        unit: "count",
        description: "First test metric.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: TEST_METRIC_2,
        name: "test_metric_2",
        unit: "ns",
        description: "Second test metric.",
        payload_info: "N/A",
    },
];

// ── Harness ───────────────────────────────────────────────────────────────────

pub fn registry() -> Registry {
    let mut registry = Registry::new();
    registry.build_scopes(TEST_SCOPES);
    registry.build_metrics(TEST_METRICS);
    registry
}

pub struct Harness<'a> {
    pub channel: Channel,
    pub subscriber: Subscriber<'a>,
    pub records: Arc<Mutex<Vec<ResolvedRecord>>>,
}

pub fn harness(registry: &Registry) -> Harness<'_> {
    let channel = Channel::unbounded();
    let writer = MemoryWriter::default();
    let records = writer.records.clone();

    let mut subscriber = Subscriber::new(channel.receiver.clone(), registry);
    subscriber.add_writer(writer);

    Harness {
        channel,
        subscriber,
        records,
    }
}

// ── Assertions Helpers ────────────────────────────────────────────────────────

pub fn find_metric(record: &ResolvedRecord, id: MetricId) -> Option<&ResolvedMetric> {
    record.metrics.iter().find(|m| m.metric_id == id)
}

pub fn has_metric(record: &ResolvedRecord, id: MetricId) -> bool {
    find_metric(record, id).is_some()
}
