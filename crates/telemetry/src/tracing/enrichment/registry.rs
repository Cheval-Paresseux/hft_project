use super::metadata::{MetricMetadata, ScopeMetadata};

use crate::schema::{metrics::TELEMETRY_METRICS, scopes::TELEMETRY_SCOPES};
use crate::tracing::capture::{MetricId, ScopeId};

use std::collections::HashMap;

// ── Registry ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Registry {
    scope_map: HashMap<ScopeId, ScopeMetadata>,
    metric_map: HashMap<MetricId, MetricMetadata>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new().with_telemetry()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self {
            scope_map: HashMap::new(),
            metric_map: HashMap::new(),
        }
    }

    pub fn with_telemetry(mut self) -> Self {
        self.build_metrics(TELEMETRY_METRICS);
        self.build_scopes(TELEMETRY_SCOPES);

        self
    }

    pub fn get_scope_metadata(&self, id: ScopeId) -> Option<&ScopeMetadata> {
        self.scope_map.get(&id)
    }

    pub fn get_metric_metadata(&self, id: MetricId) -> Option<&MetricMetadata> {
        self.metric_map.get(&id)
    }
}

// ── Builders ──────────────────────────────────────────────────────────────────

impl Registry {
    pub fn build_metrics(&mut self, metadata: &[MetricMetadata]) {
        self.metric_map.reserve(metadata.len());

        for entry in metadata {
            self.register_metric(*entry);
        }
    }

    pub fn build_scopes(&mut self, metadata: &[ScopeMetadata]) {
        self.scope_map.reserve(metadata.len());

        for entry in metadata {
            self.register_scope(*entry);
        }
    }

    fn register_metric(&mut self, metadata: MetricMetadata) {
        let previous = self.metric_map.insert(metadata.id, metadata);

        debug_assert!(
            previous.is_none(),
            "Metric ID collision detected: {:?}",
            metadata.id
        );
    }

    fn register_scope(&mut self, metadata: ScopeMetadata) {
        let previous = self.scope_map.insert(metadata.id, metadata);

        debug_assert!(
            previous.is_none(),
            "Scope ID collision detected: {:?}",
            metadata.id
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_registry_is_empty() {
        let registry = Registry::new();

        assert!(registry.scope_map.is_empty());
        assert!(registry.metric_map.is_empty());
    }

    #[test]
    fn telemetry_registry_contains_default_metadata() {
        let registry = Registry::default();

        for metric in TELEMETRY_METRICS {
            assert!(
                registry.get_metric_metadata(metric.id).is_some(),
                "Missing metric {:?}",
                metric.id
            );
        }

        for scope in TELEMETRY_SCOPES {
            assert!(
                registry.get_scope_metadata(scope.id).is_some(),
                "Missing scope {:?}",
                scope.id
            );
        }
    }

    #[test]
    fn register_and_retrieve_metric() {
        let mut registry = Registry::new();

        let metadata = MetricMetadata {
            id: MetricId::new(0x1201, 42),
            name: "test_name",
            unit: "test_unit",
            description: "test_description",
            payload_info: "test_payload",
        };

        registry.build_metrics(&[metadata]);

        let retrieved = registry
            .get_metric_metadata(metadata.id)
            .expect("metric should exist");

        assert_eq!(retrieved.id, metadata.id);
    }

    #[test]
    fn register_and_retrieve_scope() {
        let mut registry = Registry::new();

        let metadata = ScopeMetadata {
            id: ScopeId::new(0x1201, 42),
            name: "test_name",
            description: "test_description",
            payload_info: "test_payload",
        };

        registry.build_scopes(&[metadata]);

        let retrieved = registry
            .get_scope_metadata(metadata.id)
            .expect("scope should exist");

        assert_eq!(retrieved.id, metadata.id);
    }

    #[test]
    fn missing_metric_returns_none() {
        let registry = Registry::new();

        let result = registry.get_metric_metadata(MetricId::new(0x1201, 999));

        assert!(result.is_none());
    }

    #[test]
    fn missing_scope_returns_none() {
        let registry = Registry::new();

        let result = registry.get_scope_metadata(ScopeId::new(0x1201, 999));

        assert!(result.is_none());
    }
}
