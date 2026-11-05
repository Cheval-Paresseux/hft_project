use super::writer::Writer;

use crate::tracing::enrichment::{ResolvedMetric, ResolvedRecord};

use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Transaction};
use std::future::Future;
use std::thread;
use tokio::sync::{mpsc, oneshot};

// ── Postgres Writer ─────────────────────────────────────────────────────────

pub struct PostgresWriter {
    tx: mpsc::UnboundedSender<Msg>,
}

impl Writer for PostgresWriter {
    fn write(&mut self, records: &[ResolvedRecord]) {
        if self.tx.send(Msg::Records(records.to_vec())).is_err() {
            eprintln!(
                "postgres writer worker is gone; dropping {} records",
                records.len()
            );
        }
    }
}

// ── Store ───────────────────────────────────────────────────────────────────

trait Store {
    async fn write_record(&mut self, record: &ResolvedRecord) -> Result<(), sqlx::Error>;
}

struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new().connect(url).await?;

        Ok(Self { pool })
    }
}

impl Store for PostgresStore {
    async fn write_record(&mut self, record: &ResolvedRecord) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        Self::insert_record(record, &mut tx).await?;

        for metric in &record.metrics {
            Self::insert_metric(metric, &mut tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

// ── Insertion ───────────────────────────────────────────────────────────────

impl PostgresStore {
    async fn insert_record(
        record: &ResolvedRecord,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO telemetry_scope (
                instance_uuid,
                parent_uuid,
                scope_id,
                namespace_id,
                scope_name,
                payload,
                payload_info
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(record.instance_uuid)
        .bind(record.parent_uuid)
        .bind(record.scope_id.id as i32)
        .bind(record.scope_id.namespace as i32)
        .bind(record.scope_name)
        .bind(record.payload as i64)
        .bind(record.payload_info)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn insert_metric(
        metric: &ResolvedMetric,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO telemetry_metric (
                instance_uuid,
                metric_id,
                namespace_id,
                name,
                unit,
                value,
                payload,
                payload_info
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(metric.instance_uuid)
        .bind(metric.metric_id.id as i32)
        .bind(metric.metric_id.namespace as i32)
        .bind(metric.name)
        .bind(metric.unit)
        .bind(i64::try_from(metric.value).expect("metric value exceeds i64::MAX"))
        .bind(metric.payload as i64)
        .bind(metric.payload_info)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

// ── Run logic ───────────────────────────────────────────────────────────────

impl PostgresWriter {
    pub async fn new(url: &str) -> (Self, PostgresGuard) {
        let url = url.to_owned();

        Self::spawn_worker(move || async move { PostgresStore::connect(&url).await }).await
    }

    async fn spawn_worker<S, F, Fut>(connect: F) -> (Self, PostgresGuard)
    where
        S: Store + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<S, sqlx::Error>>,
    {
        let (tx, rx) = mpsc::unbounded_channel::<Msg>();
        let (ready_tx, ready_rx) = oneshot::channel();

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to build writer runtime");

            rt.block_on(async move {
                let store = match connect().await {
                    Ok(store) => store,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                let _ = ready_tx.send(Ok(()));
                Self::run(store, rx).await;
            });
        });

        ready_rx
            .await
            .expect("writer thread died before connecting")
            .expect("failed to connect to PostgreSQL");

        let writer_tx = tx.clone();
        (
            Self { tx: writer_tx },
            PostgresGuard {
                tx,
                handle: Some(handle),
            },
        )
    }

    async fn run<S: Store>(mut store: S, mut rx: mpsc::UnboundedReceiver<Msg>) {
        while let Some(msg) = rx.recv().await {
            match msg {
                Msg::Records(records) => {
                    for record in &records {
                        if let Err(e) = store.write_record(record).await {
                            eprintln!("failed to write telemetry record: {e}");
                        }
                    }
                }
                Msg::Shutdown(ack) => {
                    let _ = ack.send(());
                    break;
                }
            }
        }
    }
}

// ── Messages to the background worker ───────────────────────────────────────

enum Msg {
    Records(Vec<ResolvedRecord>),
    Shutdown(oneshot::Sender<()>),
}

// ── Lifecycle Guard ─────────────────────────────────────────────────────────

pub struct PostgresGuard {
    tx: mpsc::UnboundedSender<Msg>,
    handle: Option<thread::JoinHandle<()>>,
}

impl PostgresGuard {
    pub async fn shutdown(mut self) {
        let (ack_tx, ack_rx) = oneshot::channel();

        if self.tx.send(Msg::Shutdown(ack_tx)).is_ok() {
            let _ = ack_rx.await;
        }

        if let Some(handle) = self.handle.take() {
            let _ = tokio::task::spawn_blocking(move || handle.join()).await;
        }
    }
}

impl Drop for PostgresGuard {
    fn drop(&mut self) {
        let (ack_tx, ack_rx) = oneshot::channel();

        if self.tx.send(Msg::Shutdown(ack_tx)).is_ok() {
            let _ = ack_rx.blocking_recv();
        }

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::capture::{MetricId, ScopeId};
    use crate::tracing::enrichment::ResolvedMetric;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    #[derive(Clone, Default)]
    struct FakeStore {
        records: Arc<Mutex<Vec<ResolvedRecord>>>,
        calls: Arc<Mutex<usize>>,
        fail: Arc<Mutex<bool>>,
    }

    impl Store for FakeStore {
        async fn write_record(&mut self, record: &ResolvedRecord) -> Result<(), sqlx::Error> {
            *self.calls.lock().unwrap() += 1;

            if *self.fail.lock().unwrap() {
                return Err(sqlx::Error::RowNotFound);
            }

            self.records.lock().unwrap().push(record.clone());

            Ok(())
        }
    }

    fn record(scope_id: u16) -> ResolvedRecord {
        let instance_uuid = Uuid::new_v4();

        ResolvedRecord {
            instance_uuid,
            parent_uuid: None,

            scope_id: ScopeId::new(0x1201, scope_id),
            scope_name: "test_scope",
            metrics: vec![ResolvedMetric {
                instance_uuid,
                metric_id: MetricId::new(0x1201, 1),
                name: "test_metric",
                unit: "count",
                value: 1,
                payload: 0,
                payload_info: "N/A",
            }],

            payload: 0,
            payload_info: "N/A",
        }
    }

    #[tokio::test]
    #[should_panic(expected = "failed to connect to PostgreSQL")]
    async fn new() {
        let _ = PostgresWriter::new("not-a-valid-postgres-url").await;
    }

    #[tokio::test]
    async fn spawn_worker() {
        let store = FakeStore::default();
        let records = store.records.clone();

        let (mut writer, guard) =
            PostgresWriter::spawn_worker(move || async move { Ok(store) }).await;

        let record = record(1);
        Writer::write(&mut writer, std::slice::from_ref(&record));

        guard.shutdown().await;

        assert_eq!(*records.lock().unwrap(), vec![record]);
    }

    #[tokio::test]
    async fn run() {
        let store = FakeStore::default();
        let calls = store.calls.clone();
        *store.fail.lock().unwrap() = true;

        let (tx, rx) = mpsc::unbounded_channel();
        tx.send(Msg::Records(vec![record(1), record(2)])).unwrap();
        let (ack_tx, ack_rx) = oneshot::channel();
        tx.send(Msg::Shutdown(ack_tx)).unwrap();
        std::mem::drop(tx);

        PostgresWriter::run(store, rx).await;

        assert_eq!(*calls.lock().unwrap(), 2);
        ack_rx.await.unwrap();

        // A failing store logs the error but must not stop processing messages.
        let store = FakeStore::default();
        let records = store.records.clone();

        let expected = record(3);

        let (tx, rx) = mpsc::unbounded_channel();
        tx.send(Msg::Records(vec![expected.clone()])).unwrap();
        let (ack_tx, _ack_rx) = oneshot::channel();
        tx.send(Msg::Shutdown(ack_tx)).unwrap();
        std::mem::drop(tx);

        PostgresWriter::run(store, rx).await;

        assert_eq!(*records.lock().unwrap(), vec![expected]);
    }

    #[test]
    fn write() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut writer = PostgresWriter { tx };

        let first = record(1);
        let second = record(2);

        Writer::write(&mut writer, &[first.clone(), second.clone()]);

        match rx.try_recv().unwrap() {
            Msg::Records(records) => assert_eq!(records, vec![first, second]),
            Msg::Shutdown(_) => panic!("unexpected shutdown message"),
        }
    }

    #[tokio::test]
    async fn shutdown() {
        let (tx, mut rx) = mpsc::unbounded_channel();

        let worker = tokio::spawn(async move {
            match rx.recv().await {
                Some(Msg::Shutdown(ack)) => ack.send(()).unwrap(),
                _ => panic!("expected shutdown message"),
            }
        });

        let guard = PostgresGuard { tx, handle: None };
        guard.shutdown().await;

        worker.await.unwrap();
    }

    #[test]
    fn drop() {
        let (tx, mut rx) = mpsc::unbounded_channel();

        let worker = thread::spawn(move || match rx.blocking_recv() {
            Some(Msg::Shutdown(ack)) => ack.send(()).unwrap(),
            _ => panic!("expected shutdown message"),
        });

        let guard = PostgresGuard { tx, handle: None };
        std::mem::drop(guard);

        worker.join().unwrap();
    }
}
