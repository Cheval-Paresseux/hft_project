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
