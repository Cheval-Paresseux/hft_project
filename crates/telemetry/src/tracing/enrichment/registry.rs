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
    use super::{MetricMetadata, Registry, ScopeMetadata, TELEMETRY_METRICS, TELEMETRY_SCOPES};
    use crate::tracing::capture::{MetricId, ScopeId};

    const TEST_NAMESPACE: u16 = 0x9999;

    fn metric(id: u16, name: &'static str) -> MetricMetadata {
        MetricMetadata {
            id: MetricId::new(TEST_NAMESPACE, id),
            name,
            unit: "unit",
            description: "description",
            payload_info: "payload",
        }
    }

    fn scope(id: u16, name: &'static str) -> ScopeMetadata {
        ScopeMetadata {
            id: ScopeId::new(TEST_NAMESPACE, id),
            name,
            description: "description",
            payload_info: "payload",
        }
    }

    fn assert_contains_all_telemetry(registry: &Registry) {
        for expected in TELEMETRY_METRICS {
            assert_eq!(
                registry.get_metric_metadata(expected.id),
                Some(expected),
                "missing metric {:?}",
                expected.id
            );
        }

        for expected in TELEMETRY_SCOPES {
            assert_eq!(
                registry.get_scope_metadata(expected.id),
                Some(expected),
                "missing scope {:?}",
                expected.id
            );
        }
    }

    #[test]
    fn new() {
        let registry = Registry::new();

        assert!(registry.scope_map.is_empty());
        assert!(registry.metric_map.is_empty());
        assert_eq!(
            registry.get_scope_metadata(ScopeId::new(TEST_NAMESPACE, 1)),
            None
        );
        assert_eq!(
            registry.get_metric_metadata(MetricId::new(TEST_NAMESPACE, 1)),
            None
        );
    }

    #[test]
    fn default() {
        let registry = Registry::default();

        assert_eq!(registry.metric_map.len(), TELEMETRY_METRICS.len());
        assert_eq!(registry.scope_map.len(), TELEMETRY_SCOPES.len());
        assert_contains_all_telemetry(&registry);
    }

    #[test]
    fn with_telemetry() {
        let custom_metric = metric(1, "custom_metric");
        let custom_scope = scope(1, "custom_scope");

        let mut registry = Registry::new();
        registry.build_metrics(&[custom_metric]);
        registry.build_scopes(&[custom_scope]);

        let registry = registry.with_telemetry();

        assert_contains_all_telemetry(&registry);
        assert_eq!(
            registry.get_metric_metadata(custom_metric.id),
            Some(&custom_metric)
        );
        assert_eq!(
            registry.get_scope_metadata(custom_scope.id),
            Some(&custom_scope)
        );
    }

    #[test]
    fn get_metric_metadata() {
        let expected = metric(42, "metric");

        let mut registry = Registry::new();
        registry.build_metrics(&[expected]);

        assert_eq!(registry.get_metric_metadata(expected.id), Some(&expected));
        assert_eq!(
            registry.get_metric_metadata(MetricId::new(TEST_NAMESPACE, 43)),
            None
        );
    }

    #[test]
    fn get_scope_metadata() {
        let expected = scope(42, "scope");

        let mut registry = Registry::new();
        registry.build_scopes(&[expected]);

        assert_eq!(registry.get_scope_metadata(expected.id), Some(&expected));
        assert_eq!(
            registry.get_scope_metadata(ScopeId::new(TEST_NAMESPACE, 43)),
            None
        );
    }

    #[test]
    fn build_metrics() {
        let metadata = [metric(1, "first"), metric(2, "second"), metric(3, "third")];

        let mut registry = Registry::new();
        registry.build_metrics(&metadata);

        assert_eq!(registry.metric_map.len(), metadata.len());

        for expected in &metadata {
            assert_eq!(registry.get_metric_metadata(expected.id), Some(expected));
        }
    }

    #[test]
    fn build_scopes() {
        let metadata = [scope(1, "first"), scope(2, "second"), scope(3, "third")];

        let mut registry = Registry::new();
        registry.build_scopes(&metadata);

        assert_eq!(registry.scope_map.len(), metadata.len());

        for expected in &metadata {
            assert_eq!(registry.get_scope_metadata(expected.id), Some(expected));
        }
    }
}
