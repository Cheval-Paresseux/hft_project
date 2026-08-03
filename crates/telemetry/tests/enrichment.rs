mod common;

use common::{
    find_metric, harness, registry, TEST_METRIC_1, TEST_METRIC_2, TEST_SCOPE_1, TEST_SCOPE_2,
};
use telemetry::tracing::capture::{MetricId, Scope, ScopeId};

// ── Test ──────────────────────────────────────────────────────────────────────

#[test]
fn registered_ids_resolve_to_metadata() {
    let registry = registry();
    let mut ctx = harness(&registry);

    {
        let mut scope = Scope::init(TEST_SCOPE_1, ctx.channel.sender, None);
        scope.metric(TEST_METRIC_1, 42, None);
    }

    ctx.subscriber.drain();

    let records = ctx.records.lock().unwrap();
    let record = &records[0];

    assert_eq!(record.scope_name, "test_scope_1");

    let resolved = find_metric(record, TEST_METRIC_1).expect("metric should be present");
    assert_eq!(resolved.name, "test_metric_1");
    assert_eq!(resolved.unit, "count");
}

#[test]
fn unknown_ids_resolve_to_na() {
    let registry = registry();
    let mut ctx = harness(&registry);

    let unknown_scope = ScopeId::new(0x999, 0);
    let unknown_metric = MetricId::new(0x999, 0);

    {
        let mut scope = Scope::init(unknown_scope, ctx.channel.sender, None);
        scope.metric(unknown_metric, 42, None);
    }

    ctx.subscriber.drain();

    let records = ctx.records.lock().unwrap();
    let record = &records[0];

    assert_eq!(record.scope_name, "N/A");

    let resolved = find_metric(record, unknown_metric).expect("metric should be present");
    assert_eq!(resolved.name, "N/A");
    assert_eq!(resolved.unit, "N/A");
}

#[test]
fn metric_payload_is_forwarded() {
    let registry = registry();
    let mut ctx = harness(&registry);

    {
        let mut scope = Scope::init(TEST_SCOPE_1, ctx.channel.sender, None);
        scope.metric(TEST_METRIC_2, 102, Some(400));
    }

    ctx.subscriber.drain();

    let records = ctx.records.lock().unwrap();
    let resolved = find_metric(&records[0], TEST_METRIC_2).expect("metric should be present");

    assert_eq!(resolved.value, 102);
    assert_eq!(resolved.payload, 400);
}

#[test]
fn children_link_to_their_parent() {
    let registry = registry();
    let mut ctx = harness(&registry);

    {
        let root = Scope::init(TEST_SCOPE_1, ctx.channel.sender, None);
        let child = root.child(TEST_SCOPE_2, None);

        drop(child);
        drop(root);
    }

    ctx.subscriber.drain();

    let records = ctx.records.lock().unwrap();
    let root_record = records
        .iter()
        .find(|r| r.parent_uuid.is_none())
        .expect("root record should exist");
    let child_record = records
        .iter()
        .find(|r| r.parent_uuid.is_some())
        .expect("child record should exist");

    assert_eq!(child_record.parent_uuid, Some(root_record.instance_uuid));
}
