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
    use super::{Metric, Record};

    mod metric {
        use crate::tracing::capture::{Metric, MetricId};

        #[test]
        fn new() {
            let metric_id = MetricId::new(0x1201, 42);

            let metric = Metric::new(metric_id, 123, 7);

            assert_eq!(metric.metric_id, metric_id);
            assert_eq!(metric.value, 123);
            assert_eq!(metric.payload, 7);
        }
    }

    mod record {
        use crate::tracing::capture::{Metric, MetricId, Record, ScopeId};

        #[test]
        fn new() {
            let scope_id = ScopeId::new(0x1201, 1);

            let record = Record::new(7, Some(3), scope_id, 99);

            assert_eq!(record.instance_id, 7);
            assert_eq!(record.parent_id, Some(3));
            assert_eq!(record.scope_id, scope_id);
            assert!(record.metrics.is_empty());
            assert_eq!(record.payload, 99);
        }

        #[test]
        fn add_metric() {
            let mut record = Record::new(7, None, ScopeId::new(0x1201, 1), 0);

            for id in 0..14 {
                record.add_metric(Metric::new(MetricId::new(0x1201, id), id as u64, 0));
            }

            assert_eq!(record.metrics.len(), 14);

            for (id, metric) in record.metrics.iter().enumerate() {
                assert_eq!(metric.metric_id, MetricId::new(0x1201, id as u16));
                assert_eq!(metric.value, id as u64);
            }
        }
    }

    #[test]
    fn metric_and_record_have_expected_layout() {
        assert_eq!(std::mem::size_of::<Metric>(), 16);

        #[cfg(target_pointer_width = "64")]
        assert_eq!(std::mem::size_of::<Record>(), 256);
    }
}
