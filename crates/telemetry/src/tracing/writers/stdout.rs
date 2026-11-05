use super::writer::Writer;

use crate::tracing::enrichment::ResolvedRecord;

use std::io::{self, Write};

// ── Standard Output Writer ────────────────────────────────────────────────────

#[derive(Default)]
pub struct StdoutWriter;

impl Writer for StdoutWriter {
    fn write(&mut self, records: &[ResolvedRecord]) {
        let stdout = io::stdout();

        self.write_to(records, &mut stdout.lock()).unwrap();
    }
}

// ── Printing ──────────────────────────────────────────────────────────────────

impl StdoutWriter {
    fn write_to<W: Write>(&mut self, records: &[ResolvedRecord], out: &mut W) -> io::Result<()> {
        for record in records {
            self.print_record(record, out)?;
        }

        Ok(())
    }

    fn print_record<W: Write>(&mut self, record: &ResolvedRecord, out: &mut W) -> io::Result<()> {
        writeln!(
            out,
            "[Scope {} - {} | Instance {} | Parent {:?} | Payload {} - {}]",
            record.scope_id.id,
            record.scope_name,
            record.instance_uuid,
            record.parent_uuid,
            record.payload,
            record.payload_info,
        )?;

        for metric in &record.metrics {
            writeln!(
                out,
                "  ├─ Metric {} - {} | Value {} {} | Payload {} - {}",
                metric.metric_id.id,
                metric.name,
                metric.value,
                metric.unit,
                metric.payload,
                metric.payload_info,
            )?;
        }

        writeln!(out)?;

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::StdoutWriter;
    use crate::tracing::capture::{MetricId, ScopeId};
    use crate::tracing::enrichment::{ResolvedMetric, ResolvedRecord};
    use crate::tracing::writers::Writer;
    use uuid::Uuid;

    fn record() -> ResolvedRecord {
        let instance_uuid = Uuid::new_v4();

        ResolvedRecord {
            instance_uuid,
            parent_uuid: None,

            scope_id: ScopeId::new(0x1201, 1),
            scope_name: "test_scope",
            metrics: vec![
                ResolvedMetric {
                    instance_uuid,
                    metric_id: MetricId::new(0x1201, 10),
                    name: "first",
                    unit: "ns",
                    value: 123,
                    payload: 7,
                    payload_info: "payload",
                },
                ResolvedMetric {
                    instance_uuid,
                    metric_id: MetricId::new(0x1201, 11),
                    name: "second",
                    unit: "count",
                    value: 456,
                    payload: 0,
                    payload_info: "N/A",
                },
            ],

            payload: 99,
            payload_info: "scope_payload",
        }
    }

    #[test]
    fn default() {
        let _writer: StdoutWriter = Default::default();
    }

    #[test]
    fn write() {
        // Exercises the real stdout path without emitting any records.
        Writer::write(&mut StdoutWriter, &[]);
    }

    #[test]
    fn write_to() {
        let record = record();
        let mut output = Vec::new();

        StdoutWriter
            .write_to(std::slice::from_ref(&record), &mut output)
            .unwrap();

        let first = &record.metrics[0];
        let second = &record.metrics[1];

        let expected = [
            format!(
                "[Scope {} - {} | Instance {} | Parent {:?} | Payload {} - {}]",
                record.scope_id.id,
                record.scope_name,
                record.instance_uuid,
                record.parent_uuid,
                record.payload,
                record.payload_info,
            ),
            format!(
                "  ├─ Metric {} - {} | Value {} {} | Payload {} - {}",
                first.metric_id.id,
                first.name,
                first.value,
                first.unit,
                first.payload,
                first.payload_info,
            ),
            format!(
                "  ├─ Metric {} - {} | Value {} {} | Payload {} - {}",
                second.metric_id.id,
                second.name,
                second.value,
                second.unit,
                second.payload,
                second.payload_info,
            ),
            String::new(),
            String::new(),
        ]
        .join("\n");

        assert_eq!(String::from_utf8(output).unwrap(), expected);

        let mut empty = Vec::new();
        StdoutWriter.write_to(&[], &mut empty).unwrap();
        assert!(empty.is_empty());
    }
}
