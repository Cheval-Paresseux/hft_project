use super::ids::{MetricId, ScopeId, ScopeInstanceId, next_id};
use super::record::{Metric, MetricPayload, MetricValue, Record, ScopePayload};

use crate::collectors::time::CLOCK;
use crate::collectors::*;
use crate::schema::metrics::*;

use crossbeam::channel::Sender;

// ── Scope ─────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct Scope {
    pub scope_id: ScopeId,
    pub instance_id: ScopeInstanceId,
    pub parent_id: Option<ScopeInstanceId>,
    pub payload: ScopePayload,

    sender: Sender<Record>,
    record: Option<Record>,
}

impl Scope {
    fn new(
        scope_id: ScopeId,
        parent_id: Option<ScopeInstanceId>,
        sender: Sender<Record>,
        payload: ScopePayload,
    ) -> Self {
        let mut scope = Self {
            scope_id,
            instance_id: next_id(),
            parent_id,
            payload,

            sender,
            record: None,
        };

        scope.scope_start();

        scope
    }

    pub fn init(scope_id: ScopeId, sender: Sender<Record>, payload: Option<ScopePayload>) -> Self {
        Scope::new(scope_id, None, sender, payload.unwrap_or_default())
    }

    pub fn child(&self, scope_id: ScopeId, payload: Option<ScopePayload>) -> Self {
        Scope::new(
            scope_id,
            Some(self.instance_id),
            self.sender.clone(),
            payload.unwrap_or_default(),
        )
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        self.scope_end();
        self.flush();
    }
}

// ── Metrics management ────────────────────────────────────────────────────────

impl Scope {
    pub fn metric(&mut self, id: MetricId, value: MetricValue, payload: Option<MetricPayload>) {
        let record = self.record.get_or_insert_with(|| {
            Record::new(
                self.instance_id,
                self.parent_id,
                self.scope_id,
                self.payload,
            )
        });

        record.add_metric(Metric::new(id, value, payload.unwrap_or_default()));
    }

    pub fn flush(&mut self) {
        let Some(record) = self.record.take() else {
            return;
        };

        let _ = self.sender.try_send(record);
    }
}

// ── Standard Metrics ──────────────────────────────────────────────────────────

impl Scope {
    fn scope_start(&mut self) {
        self.metric(SCOPE_START_TIME, CLOCK.unix_nanos(), None);
    }

    fn scope_end(&mut self) {
        self.metric(SCOPE_END_TIME, CLOCK.unix_nanos(), None);
    }
}

impl Scope {
    pub fn start(&mut self) {
        self.metric(START_TIME, CLOCK.unix_nanos(), None);
    }

    pub fn end(&mut self) {
        self.metric(END_TIME, CLOCK.unix_nanos(), None);
    }

    pub fn events(&mut self, count: MetricValue) {
        self.metric(EVENTS, count, None);
    }

    pub fn errors(&mut self, count: MetricValue) {
        self.metric(ERRORS, count, None);
    }

    pub fn warnings(&mut self, count: MetricValue) {
        self.metric(WARNINGS, count, None);
    }

    pub fn global_allocations_snapshot(&mut self) {
        let snapshot = global_allocations_snapshot();

        self.metric(MEMORY_ALLOCATIONS, snapshot.allocations, None);
        self.metric(MEMORY_DEALLOCATIONS, snapshot.deallocations, None);
        self.metric(MEMORY_REALLOCATIONS, snapshot.reallocations, None);
        self.metric(MEMORY_ALLOCATED, snapshot.allocated_bytes, None);
        self.metric(MEMORY_DEALLOCATED, snapshot.deallocated_bytes, None);
    }

    pub fn process_memory_snapshot(&mut self) {
        let process_memory: ProcessMemory = process_memory_snapshot();

        self.metric(RESIDENT_MEMORY, process_memory.resident, None);
        self.metric(RESIDENT_MEMORY_PEAK, process_memory.resident_peak, None);
        self.metric(VIRTUAL_MEMORY, process_memory.virtual_, None);
        self.metric(VIRTUAL_MEMORY_PEAK, process_memory.virtual_peak, None);
        self.metric(SWAP_MEMORY, process_memory.swap, None);
        self.metric(LOCKED_MEMORY, process_memory.locked, None);
    }

    pub fn process_cpu_time_snapshot(&mut self) {
        let process_cpu_time = process_cpu_time_snapshot();

        self.metric(PROCESS_USER_CPU_TIME, process_cpu_time.user_time, None);
        self.metric(PROCESS_SYSTEM_CPU_TIME, process_cpu_time.system_time, None);
    }

    pub fn items_processed(&mut self, count: MetricValue) {
        self.metric(ITEMS_PROCESSED, count, None);
    }

    pub fn bytes_processed(&mut self, count: MetricValue) {
        self.metric(BYTES_PROCESSED, count, None);
    }

    pub fn value(&mut self, value: MetricValue) {
        self.metric(VALUE, value, None);
    }
}

impl Scope {
    pub fn memory_allocations(&mut self) {
        self.metric(MEMORY_ALLOCATIONS, allocations(), None);
    }

    pub fn memory_deallocations(&mut self) {
        self.metric(MEMORY_DEALLOCATIONS, deallocations(), None);
    }

    pub fn memory_reallocations(&mut self) {
        self.metric(MEMORY_REALLOCATIONS, reallocations(), None);
    }

    pub fn memory_allocated(&mut self) {
        self.metric(MEMORY_ALLOCATED, allocated_bytes(), None);
    }

    pub fn memory_deallocated(&mut self) {
        self.metric(MEMORY_DEALLOCATED, deallocated_bytes(), None);
    }

    pub fn resident_memory(&mut self) {
        self.metric(RESIDENT_MEMORY, process_resident_memory(), None);
    }

    pub fn resident_memory_peak(&mut self) {
        self.metric(RESIDENT_MEMORY_PEAK, process_resident_memory_peak(), None);
    }

    pub fn virtual_memory(&mut self) {
        self.metric(VIRTUAL_MEMORY, process_virtual_memory(), None);
    }

    pub fn virtual_memory_peak(&mut self) {
        self.metric(VIRTUAL_MEMORY_PEAK, process_virtual_memory_peak(), None);
    }

    pub fn swap_memory(&mut self) {
        self.metric(SWAP_MEMORY, process_swap_memory(), None);
    }

    pub fn locked_memory(&mut self) {
        self.metric(LOCKED_MEMORY, process_locked_memory(), None);
    }

    pub fn process_user_cpu_time(&mut self) {
        self.metric(PROCESS_USER_CPU_TIME, process_user_cpu_time(), None);
    }

    pub fn process_system_cpu_time(&mut self) {
        self.metric(PROCESS_SYSTEM_CPU_TIME, process_system_cpu_time(), None);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::Scope;
    use crate::schema::metrics::{SCOPE_END_TIME, SCOPE_START_TIME};
    use crate::tracing::capture::{Metric, MetricId, Record, ScopeId};
    use crossbeam::channel::{self, Receiver, Sender};

    fn test_channel() -> (Sender<Record>, Receiver<Record>) {
        channel::unbounded()
    }

    fn record_after<F>(f: F) -> Record
    where
        F: FnOnce(&mut Scope),
    {
        let (sender, receiver) = test_channel();
        let mut scope = Scope::init(ScopeId::new(0x1201, 1), sender, None);

        f(&mut scope);

        std::mem::drop(scope);

        receiver.recv().unwrap()
    }

    fn find_metric(record: &Record, id: MetricId) -> &Metric {
        record
            .metrics
            .iter()
            .find(|metric| metric.metric_id == id)
            .unwrap_or_else(|| panic!("missing metric {id:?}"))
    }

    // ── Lifecycle ─────────────────────────────────────────────────────────────

    #[test]
    fn init() {
        let (sender, receiver) = test_channel();

        let scope = Scope::init(ScopeId::new(0x1201, 1), sender, Some(42));
        let instance_id = scope.instance_id;

        assert_eq!(scope.scope_id, ScopeId::new(0x1201, 1));
        assert_eq!(scope.parent_id, None);
        assert_eq!(scope.payload, 42);

        std::mem::drop(scope);

        let record = receiver.recv().unwrap();

        assert_eq!(record.instance_id, instance_id);
        assert_eq!(record.scope_id, ScopeId::new(0x1201, 1));
        assert_eq!(record.parent_id, None);
        assert_eq!(record.payload, 42);
        assert!(
            record
                .metrics
                .iter()
                .any(|m| m.metric_id == SCOPE_START_TIME)
        );
        assert!(record.metrics.iter().any(|m| m.metric_id == SCOPE_END_TIME));

        let (sender, receiver) = test_channel();
        let scope = Scope::init(ScopeId::new(0x1201, 2), sender, None);

        std::mem::drop(scope);

        assert_eq!(receiver.recv().unwrap().payload, 0);
    }

    #[test]
    fn child() {
        let (sender, receiver) = test_channel();

        let parent = Scope::init(ScopeId::new(0x1201, 1), sender, None);
        let parent_instance_id = parent.instance_id;

        let child = parent.child(ScopeId::new(0x1201, 2), Some(9));
        let child_instance_id = child.instance_id;

        assert_eq!(child.parent_id, Some(parent_instance_id));
        assert_eq!(child.scope_id, ScopeId::new(0x1201, 2));
        assert_eq!(child.payload, 9);

        std::mem::drop(child);
        std::mem::drop(parent);

        let records: Vec<_> = (0..2).map(|_| receiver.recv().unwrap()).collect();
        let child_record = records
            .iter()
            .find(|record| record.instance_id == child_instance_id)
            .expect("child record should be emitted");

        assert_eq!(child_record.parent_id, Some(parent_instance_id));
    }

    #[test]
    fn metric() {
        let record = record_after(|scope| {
            scope.metric(MetricId::new(0x1201, 42), 123, Some(7));
            scope.metric(MetricId::new(0x1201, 43), 456, None);
        });

        let with_payload = find_metric(&record, MetricId::new(0x1201, 42));
        assert_eq!(with_payload.value, 123);
        assert_eq!(with_payload.payload, 7);

        let default_payload = find_metric(&record, MetricId::new(0x1201, 43));
        assert_eq!(default_payload.value, 456);
        assert_eq!(default_payload.payload, 0);
    }

    #[test]
    fn flush() {
        let (sender, receiver) = test_channel();

        let mut scope = Scope::init(ScopeId::new(0x1201, 1), sender, None);

        scope.metric(MetricId::new(0x1201, 42), 123, None);
        scope.flush();

        let first = receiver.recv().unwrap();
        assert!(
            first
                .metrics
                .iter()
                .any(|m| m.metric_id == MetricId::new(0x1201, 42))
        );

        scope.flush();
        assert!(receiver.try_recv().is_err());

        scope.metric(MetricId::new(0x1201, 43), 456, None);
        scope.flush();

        let second = receiver.recv().unwrap();
        assert!(
            second
                .metrics
                .iter()
                .any(|m| m.metric_id == MetricId::new(0x1201, 43))
        );
        assert!(
            !second
                .metrics
                .iter()
                .any(|m| m.metric_id == SCOPE_START_TIME)
        );

        std::mem::drop(scope);

        let third = receiver.recv().unwrap();
        assert!(third.metrics.iter().any(|m| m.metric_id == SCOPE_END_TIME));
        assert!(
            !third
                .metrics
                .iter()
                .any(|m| m.metric_id == MetricId::new(0x1201, 43))
        );

        let (sender, receiver) = channel::bounded(0);
        let mut scope = Scope::init(ScopeId::new(0x1201, 2), sender, None);

        scope.metric(MetricId::new(0x1201, 44), 1, None);
        scope.flush();

        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn drop() {
        let (sender, receiver) = test_channel();

        {
            let _scope = Scope::init(ScopeId::new(0x1201, 1), sender, None);
        }

        let record = receiver.recv().unwrap();

        assert!(
            record
                .metrics
                .iter()
                .any(|m| m.metric_id == SCOPE_START_TIME)
        );
        assert!(record.metrics.iter().any(|m| m.metric_id == SCOPE_END_TIME));
    }
}
