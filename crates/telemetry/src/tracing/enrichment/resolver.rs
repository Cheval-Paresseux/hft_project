use super::registry::Registry;
use super::uuid_map::UUIDMap;

use crate::tracing::capture::{
    Metric, MetricId, MetricPayload, MetricValue, Record, ScopeId, ScopePayload,
};

use serde::Serialize;
use uuid::Uuid;

// ── Resolved Record ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct ResolvedMetric {
    pub instance_uuid: Uuid,

    pub metric_id: MetricId,
    pub name: &'static str,
    pub unit: &'static str,
    pub value: MetricValue,

    pub payload: MetricPayload,
    pub payload_info: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ResolvedRecord {
    pub instance_uuid: Uuid,
    pub parent_uuid: Option<Uuid>,

    pub scope_id: ScopeId,
    pub scope_name: &'static str,
    pub metrics: Vec<ResolvedMetric>,

    pub payload: ScopePayload,
    pub payload_info: &'static str,
}

// ── Resolver ──────────────────────────────────────────────────────────────────

pub struct Resolver<'a> {
    registry: &'a Registry,
    uuid_map: UUIDMap,
}

impl<'a> Resolver<'a> {
    pub fn new(registry: &'a Registry) -> Self {
        Self {
            registry,
            uuid_map: UUIDMap::default(),
        }
    }

    pub fn resolve(&mut self, record: &Record, buffer: &mut Vec<ResolvedRecord>) {
        buffer.push(self.process_record(record));
    }
}

impl<'a> Resolver<'a> {
    fn process_record(&mut self, record: &Record) -> ResolvedRecord {
        let instance_uuid = self.uuid_map.get_or_create_scope_uuid(record.instance_id);
        let parent_uuid = record
            .parent_id
            .map(|id| self.uuid_map.get_or_create_scope_uuid(id));

        let (scope_name, payload_info) = self
            .registry
            .get_scope_metadata(record.scope_id)
            .map(|scope| (scope.name, scope.payload_info))
            .unwrap_or(("N/A", "N/A"));

        let mut resolved_record = ResolvedRecord {
            instance_uuid,
            parent_uuid,

            scope_id: record.scope_id,
            scope_name,
            metrics: Vec::new(),

            payload: record.payload,
            payload_info,
        };

        for metric in &record.metrics {
            resolved_record
                .metrics
                .push(self.process_metric(*metric, instance_uuid));
        }

        resolved_record
    }

    fn process_metric(&self, metric: Metric, instance_uuid: Uuid) -> ResolvedMetric {
        let (name, unit, payload_info) = self
            .registry
            .get_metric_metadata(metric.metric_id)
            .map(|metadata| (metadata.name, metadata.unit, metadata.payload_info))
            .unwrap_or(("N/A", "N/A", "N/A"));

        ResolvedMetric {
            instance_uuid,

            metric_id: metric.metric_id,
            name,
            unit,
            value: metric.value,

            payload: metric.payload,
            payload_info,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::{ResolvedRecord, Resolver};
    use crate::schema::metrics::{EVENTS, START_TIME};
    use crate::schema::scopes::INITIALIZATION;
    use crate::tracing::capture::{Metric, MetricId, Record, ScopeId};
    use crate::tracing::enrichment::Registry;

    fn record() -> Record {
        let mut record = Record::new(7, Some(3), INITIALIZATION, 99);

        record.add_metric(Metric::new(EVENTS, 5, 0));
        record.add_metric(Metric::new(START_TIME, 123, 7));

        record
    }

    #[test]
    fn new() {
        let registry = Registry::default();

        let resolver = Resolver::new(&registry);

        assert!(std::ptr::eq(resolver.registry, &registry));
    }

    #[test]
    fn resolve() {
        let registry = Registry::default();
        let mut resolver = Resolver::new(&registry);

        let record = record();
        let mut buffer = Vec::new();

        resolver.resolve(&record, &mut buffer);

        assert_eq!(buffer.len(), 1);
        let resolved = buffer[0].clone();

        assert_eq!(resolved.scope_id, INITIALIZATION);
        assert_eq!(resolved.scope_name, "initialization");
        assert_eq!(resolved.payload, 99);
        assert_eq!(resolved.payload_info, "N/A");

        let parent_uuid = resolved
            .parent_uuid
            .expect("parent uuid should be resolved");
        assert_ne!(resolved.instance_uuid, parent_uuid);

        let events = resolved
            .metrics
            .iter()
            .find(|metric| metric.metric_id == EVENTS)
            .expect("events metric should be resolved");
        assert_eq!(events.instance_uuid, resolved.instance_uuid);
        assert_eq!(events.name, "events");
        assert_eq!(events.unit, "count");
        assert_eq!(events.value, 5);
        assert_eq!(events.payload, 0);

        let start = resolved
            .metrics
            .iter()
            .find(|metric| metric.metric_id == START_TIME)
            .expect("start metric should be resolved");
        assert_eq!(start.name, "start_time");
        assert_eq!(start.unit, "ns");
        assert_eq!(start.value, 123);
        assert_eq!(start.payload, 7);

        // Resolving the same instance again must reuse its UUIDs and append.
        resolver.resolve(&record, &mut buffer);

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer[1].instance_uuid, resolved.instance_uuid);
        assert_eq!(buffer[1].parent_uuid, Some(parent_uuid));

        // A record with the parent's instance ID resolves to the same UUID.
        let parent_record = Record::new(3, None, INITIALIZATION, 0);
        resolver.resolve(&parent_record, &mut buffer);

        assert_eq!(buffer[2].instance_uuid, parent_uuid);
        assert_eq!(buffer[2].parent_uuid, None);

        // Unknown scope and metric IDs fall back to placeholder metadata.
        let unknown = Record::new(8, None, ScopeId::new(0x9999, 1), 0);
        let mut unknown_record = unknown;
        unknown_record.add_metric(Metric::new(MetricId::new(0x9999, 1), 1, 0));
        resolver.resolve(&unknown_record, &mut buffer);

        let resolved_unknown: &ResolvedRecord = &buffer[3];
        assert_eq!(resolved_unknown.scope_name, "N/A");
        assert_eq!(resolved_unknown.payload_info, "N/A");
        assert_eq!(resolved_unknown.metrics[0].name, "N/A");
        assert_eq!(resolved_unknown.metrics[0].unit, "N/A");
        assert_eq!(resolved_unknown.metrics[0].payload_info, "N/A");
    }
}
