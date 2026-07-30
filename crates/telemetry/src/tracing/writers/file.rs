use super::writer::Writer;

use crate::tracing::enrichment::ResolvedRecord;

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Result, Write};
use std::path::Path;

// ── File Writer ──────────────────────────────────────────────────────────────

pub struct FileWriter {
    writer: BufWriter<File>,
}

impl Writer for FileWriter {
    fn write(&mut self, records: &[ResolvedRecord]) {
        for record in records {
            self.write_record(record);
        }
    }
}

// ── Path Resolver ────────────────────────────────────────────────────────────

impl FileWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path)?;

        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn existing<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self {
            writer: BufWriter::new(file),
        })
    }
}

// ── Writing ──────────────────────────────────────────────────────────────────

impl FileWriter {
    fn write_record(&mut self, record: &ResolvedRecord) {
        writeln!(
            self.writer,
            "scope, {}, {}, {}, {:?}, {}, {}",
            record.scope_id.id,
            record.scope_name,
            record.instance_uuid,
            record.parent_uuid,
            record.payload,
            record.payload_info
        )
        .unwrap();

        for metric in &record.metrics {
            writeln!(
                self.writer,
                "metric, {}, {}, {}, {}, {}, {}",
                metric.metric_id.id,
                metric.name,
                metric.value,
                metric.unit,
                metric.payload,
                metric.payload_info,
            )
            .unwrap();
        }
    }
}
