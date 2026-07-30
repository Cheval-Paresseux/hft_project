use super::writer::Writer;

use crate::tracing::enrichment::ResolvedRecord;

// ── Standard Output Writer ────────────────────────────────────────────────────

#[derive(Default)]
pub struct StdoutWriter;

impl Writer for StdoutWriter {
    fn write(&mut self, records: &[ResolvedRecord]) {
        for record in records {
            self.print_record(record);
        }
    }
}

// ── Printing ──────────────────────────────────────────────────────────────────

impl StdoutWriter {
    fn print_record(&mut self, record: &ResolvedRecord) {
        println!(
            "[Scope {} - {} | Instance {} | Parent {:?} | Payload {} - {}]",
            record.scope_id.id,
            record.scope_name,
            record.instance_uuid,
            record.parent_uuid,
            record.payload,
            record.payload_info,
        );

        for metric in &record.metrics {
            println!(
                "  ├─ Metric {} - {} | Value {} {} | Payload {} - {}",
                metric.metric_id.id,
                metric.name,
                metric.value,
                metric.unit,
                metric.payload,
                metric.payload_info,
            );
        }

        println!();
    }
}
