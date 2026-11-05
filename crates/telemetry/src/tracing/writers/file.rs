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

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::FileWriter;
    use crate::tracing::capture::{MetricId, ScopeId};
    use crate::tracing::enrichment::{ResolvedMetric, ResolvedRecord};
    use crate::tracing::writers::Writer;
    use std::path::{Path, PathBuf};
    use uuid::Uuid;

    struct TempPath(PathBuf);

    impl TempPath {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "telemetry-writer-{}-{}-{name}",
                std::process::id(),
                Uuid::new_v4()
            ));

            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempPath {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn record(parent_uuid: Option<Uuid>) -> ResolvedRecord {
        let instance_uuid = Uuid::new_v4();

        ResolvedRecord {
            instance_uuid,
            parent_uuid,

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
    fn new() {
        let path = TempPath::new("new");
        std::fs::write(path.path(), "stale content").unwrap();

        {
            let mut writer = FileWriter::new(path.path()).unwrap();
            Writer::write(&mut writer, &[record(None)]);
        }

        let contents = std::fs::read_to_string(path.path()).unwrap();

        assert!(!contents.contains("stale content"));
        assert_eq!(
            contents
                .lines()
                .filter(|line| line.starts_with("scope, "))
                .count(),
            1
        );
        assert_eq!(
            contents
                .lines()
                .filter(|line| line.starts_with("metric, "))
                .count(),
            2
        );
    }

    #[test]
    fn existing() {
        let path = TempPath::new("existing");
        std::fs::write(path.path(), "existing\n").unwrap();

        {
            let mut writer = FileWriter::existing(path.path()).unwrap();
            Writer::write(&mut writer, &[record(None)]);
        }

        {
            let mut writer = FileWriter::existing(path.path()).unwrap();
            Writer::write(&mut writer, &[record(Some(Uuid::new_v4()))]);
        }

        let contents = std::fs::read_to_string(path.path()).unwrap();

        assert!(contents.starts_with("existing\n"));
        assert_eq!(
            contents
                .lines()
                .filter(|line| line.starts_with("scope, "))
                .count(),
            2
        );

        let missing = TempPath::new("existing-missing");

        {
            let mut writer = FileWriter::existing(missing.path()).unwrap();
            Writer::write(&mut writer, &[record(None)]);
        }

        assert!(missing.path().exists());
    }

    #[test]
    fn write() {
        let path = TempPath::new("write");
        let record = record(Some(Uuid::new_v4()));

        {
            let mut writer = FileWriter::new(path.path()).unwrap();
            Writer::write(&mut writer, std::slice::from_ref(&record));
        }

        let contents = std::fs::read_to_string(path.path()).unwrap();

        let expected = format!(
            "scope, {}, {}, {}, {:?}, {}, {}\n\
             metric, {}, {}, {}, {}, {}, {}\n\
             metric, {}, {}, {}, {}, {}, {}\n",
            record.scope_id.id,
            record.scope_name,
            record.instance_uuid,
            record.parent_uuid,
            record.payload,
            record.payload_info,
            record.metrics[0].metric_id.id,
            record.metrics[0].name,
            record.metrics[0].value,
            record.metrics[0].unit,
            record.metrics[0].payload,
            record.metrics[0].payload_info,
            record.metrics[1].metric_id.id,
            record.metrics[1].name,
            record.metrics[1].value,
            record.metrics[1].unit,
            record.metrics[1].payload,
            record.metrics[1].payload_info,
        );

        assert_eq!(contents, expected);
    }
}
