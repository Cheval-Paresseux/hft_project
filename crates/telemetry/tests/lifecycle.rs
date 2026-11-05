use telemetry::schema::metrics::{SCOPE_END_TIME, SCOPE_START_TIME};
use telemetry::tracing::capture::{MetricId, Scope, ScopeId};
use telemetry::tracing::enrichment::{MetricMetadata, Registry, ScopeMetadata};
use telemetry::tracing::handoff::{Channel, Subscriber};
use telemetry::tracing::writers::{FileWriter, MemoryWriter};

// ── Test Metadata ─────────────────────────────────────────────────────────────

const ROOT: ScopeId = ScopeId::new(0x100, 0);
const CHILD: ScopeId = ScopeId::new(0x100, 1);

const METRIC: MetricId = MetricId::new(0x100, 0);
const UNKNOWN_METRIC: MetricId = MetricId::new(0x999, 1);

static SCOPES: &[ScopeMetadata] = &[
    ScopeMetadata {
        id: ROOT,
        name: "test_root",
        description: "Root test scope.",
        payload_info: "N/A",
    },
    ScopeMetadata {
        id: CHILD,
        name: "test_child",
        description: "Child test scope.",
        payload_info: "N/A",
    },
];

static METRICS: &[MetricMetadata] = &[MetricMetadata {
    id: METRIC,
    name: "test_metric",
    unit: "count",
    description: "Test metric.",
    payload_info: "N/A",
}];

// ── Temp File ─────────────────────────────────────────────────────────────────

struct TempFile(std::path::PathBuf);

impl TempFile {
    fn new(name: &str) -> Self {
        Self(std::env::temp_dir().join(format!("telemetry-{}-{}", std::process::id(), name)))
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────

#[test]
fn lifecycle() {
    let mut registry = Registry::new();
    registry.build_scopes(SCOPES);
    registry.build_metrics(METRICS);

    let channel = Channel::unbounded();

    let memory_writer = MemoryWriter::default();
    let memory_records = memory_writer.records.clone();

    let file = TempFile::new("lifecycle.csv");
    let file_writer = FileWriter::new(&file.0).unwrap();

    let mut subscriber = Subscriber::new(channel.receiver.clone(), &registry);
    subscriber.add_writer(memory_writer);
    subscriber.add_writer(file_writer);

    {
        let mut root = Scope::init(ROOT, channel.sender.clone(), Some(123));
        root.metric(METRIC, 42, Some(7));

        {
            let mut child = root.child(CHILD, Some(456));
            child.metric(UNKNOWN_METRIC, 99, None);
        }
    }

    // Closing every sender lets `run` finish once the channel is drained.
    std::mem::drop(channel.sender);
    subscriber.run();

    let records = memory_records.lock().unwrap();

    assert_eq!(records.len(), 2);

    let root_record = records
        .iter()
        .find(|record| record.scope_id == ROOT)
        .expect("root record should exist");
    let child_record = records
        .iter()
        .find(|record| record.scope_id == CHILD)
        .expect("child record should exist");

    assert_eq!(root_record.scope_name, "test_root");
    assert_eq!(root_record.payload, 123);
    assert_eq!(root_record.parent_uuid, None);
    assert!(root_record
        .metrics
        .iter()
        .any(|metric| metric.metric_id == SCOPE_START_TIME));
    assert!(root_record
        .metrics
        .iter()
        .any(|metric| metric.metric_id == SCOPE_END_TIME));

    let metric = root_record
        .metrics
        .iter()
        .find(|metric| metric.metric_id == METRIC)
        .expect("custom metric should be resolved");
    assert_eq!(metric.name, "test_metric");
    assert_eq!(metric.unit, "count");
    assert_eq!(metric.value, 42);
    assert_eq!(metric.payload, 7);

    assert_eq!(child_record.scope_name, "test_child");
    assert_eq!(child_record.parent_uuid, Some(root_record.instance_uuid));
    assert_eq!(child_record.payload, 456);

    let unknown = child_record
        .metrics
        .iter()
        .find(|metric| metric.metric_id == UNKNOWN_METRIC)
        .expect("unknown metric should still be emitted");
    assert_eq!(unknown.name, "N/A");
    assert_eq!(unknown.unit, "N/A");
    assert_eq!(unknown.value, 99);

    std::mem::drop(records);
    std::mem::drop(subscriber);

    let contents = std::fs::read_to_string(&file.0).unwrap();

    assert_eq!(
        contents
            .lines()
            .filter(|line| line.starts_with("scope, "))
            .count(),
        2
    );
    assert!(contents.lines().any(|line| line.starts_with("metric, ")));
}
