mod common;

use common::{
    find_metric, harness, has_metric, registry, TEST_METRIC_1, TEST_METRIC_2, TEST_SCOPE_1,
    TEST_SCOPE_2,
};
use telemetry::schema::metrics;
use telemetry::tracing::capture::Scope;

// ── Test ──────────────────────────────────────────────────────────────────────

#[test]
fn scope_records_its_lifecycle() {
    let registry = registry();
    let mut ctx = harness(&registry);

    {
        let mut root = Scope::init(TEST_SCOPE_1, ctx.channel.sender, None);
        root.metric(TEST_METRIC_1, 42, None);
    }

    ctx.subscriber.drain();

    let records = ctx.records.lock().unwrap();
    assert_eq!(records.len(), 1);

    let record = &records[0];
    assert_eq!(record.scope_id, TEST_SCOPE_1);
    assert!(has_metric(record, metrics::SCOPE_START_TIME));
    assert!(has_metric(record, metrics::SCOPE_END_TIME));
    assert_eq!(
        find_metric(record, TEST_METRIC_1).map(|m| m.value),
        Some(42)
    );
}

#[test]
fn nested_scopes_produce_one_record_each() {
    let registry = registry();
    let mut ctx = harness(&registry);

    {
        let mut root = Scope::init(TEST_SCOPE_1, ctx.channel.sender, None);
        root.metric(TEST_METRIC_1, 1, None);

        {
            let mut child_1 = root.child(TEST_SCOPE_2, None);
            child_1.metric(TEST_METRIC_2, 2, None);
        }

        {
            let mut child_2 = root.child(TEST_SCOPE_2, None);
            child_2.metric(TEST_METRIC_2, 3, None);
        }
    }

    ctx.subscriber.drain();

    let records = ctx.records.lock().unwrap();
    assert_eq!(records.len(), 3);
    assert_eq!(
        records.iter().filter(|r| r.parent_uuid.is_some()).count(),
        2
    );
}

#[test]
fn records_are_buffered_until_drain() {
    let registry = registry();
    let mut ctx = harness(&registry);

    let scope = Scope::init(TEST_SCOPE_1, ctx.channel.sender, None);
    drop(scope);

    assert!(ctx.records.lock().unwrap().is_empty());

    ctx.subscriber.drain();

    assert_eq!(ctx.records.lock().unwrap().len(), 1);
}
