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
