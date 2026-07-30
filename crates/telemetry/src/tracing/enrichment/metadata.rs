use crate::tracing::capture::{MetricId, ScopeId};

// ── Scope Metadata ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeMetadata {
    pub id: ScopeId,
    pub name: &'static str,
    pub description: &'static str,
    pub payload_info: &'static str,
}

// ── Metric Metadata ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetricMetadata {
    pub id: MetricId,
    pub name: &'static str,
    pub unit: &'static str,
    pub description: &'static str,
    pub payload_info: &'static str,
}
