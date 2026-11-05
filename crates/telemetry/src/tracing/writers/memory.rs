use super::writer::Writer;

use crate::tracing::enrichment::ResolvedRecord;

use std::sync::{Arc, Mutex};

// ── Standard Output Writer ────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct MemoryWriter {
    pub records: Arc<Mutex<Vec<ResolvedRecord>>>,
}

impl Writer for MemoryWriter {
    fn write(&mut self, records: &[ResolvedRecord]) {
        self.records.lock().unwrap().extend_from_slice(records);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::MemoryWriter;
    use crate::tracing::capture::ScopeId;
    use crate::tracing::enrichment::ResolvedRecord;
    use crate::tracing::writers::Writer;
    use uuid::Uuid;

    fn record(scope_id: u16) -> ResolvedRecord {
        ResolvedRecord {
            instance_uuid: Uuid::new_v4(),
            parent_uuid: None,

            scope_id: ScopeId::new(0x1201, scope_id),
            scope_name: "test_scope",
            metrics: Vec::new(),

            payload: 0,
            payload_info: "N/A",
        }
    }

    #[test]
    fn default() {
        let writer = MemoryWriter::default();

        assert!(writer.records.lock().unwrap().is_empty());
    }

    #[test]
    fn write() {
        let mut writer = MemoryWriter::default();
        let shared = writer.clone();

        let first = record(1);
        let second = record(2);

        Writer::write(&mut writer, std::slice::from_ref(&first));
        Writer::write(&mut writer, std::slice::from_ref(&second));

        assert_eq!(*shared.records.lock().unwrap(), vec![first, second]);

        Writer::write(&mut writer, &[]);
        assert_eq!(writer.records.lock().unwrap().len(), 2);
    }
}
