use super::ids::{MetricId, ScopeId, ScopeInstanceId};
use arrayvec::ArrayVec;
use serde::Serialize;

// ── Metric Log ────────────────────────────────────────────────────────────────

pub type MetricValue = u64;
pub type MetricPayload = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Metric {
    pub metric_id: MetricId,
    pub value: MetricValue,
    pub payload: MetricPayload,
}

impl Metric {
    pub fn new(metric_id: MetricId, value: MetricValue, payload: MetricPayload) -> Self {
        Self {
            metric_id,
            value,
            payload,
        }
    }
}

// ── Scope Record ──────────────────────────────────────────────────────────────

const METRIC_CAP: usize = 14;

pub type ScopePayload = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Record {
    pub instance_id: ScopeInstanceId,
    pub parent_id: Option<ScopeInstanceId>,

    pub scope_id: ScopeId,
    pub metrics: ArrayVec<Metric, METRIC_CAP>,
    pub payload: ScopePayload,
}

impl Record {
    pub fn new(
        instance_id: ScopeInstanceId,
        parent_id: Option<ScopeInstanceId>,
        scope_id: ScopeId,
        payload: ScopePayload,
    ) -> Self {
        Self {
            instance_id,
            parent_id,
            scope_id,
            metrics: ArrayVec::new(),
            payload,
        }
    }

    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.push(metric);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_log_size() {
        println!("Metric:  {} bytes", std::mem::size_of::<Metric>());
        println!("Record:  {} bytes", std::mem::size_of::<Record>());
    }
}
