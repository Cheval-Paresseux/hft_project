use super::writer::Writer;

use crate::tracing::enrichment::{ResolvedMetric, ResolvedRecord};

use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Transaction};
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

// ── Run logic ───────────────────────────────────────────────────────────────

impl PostgresWriter {
    pub async fn new(url: &str) -> (Self, PostgresGuard) {
        let url = url.to_owned();
        let (tx, rx) = mpsc::unbounded_channel::<Msg>();
        let (ready_tx, ready_rx) = oneshot::channel();

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to build writer runtime");

            rt.block_on(async move {
                let pool = match PgPoolOptions::new().connect(&url).await {
                    Ok(pool) => pool,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                let _ = ready_tx.send(Ok(()));
                Self::run(pool, rx).await;
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

    async fn run(pool: PgPool, mut rx: mpsc::UnboundedReceiver<Msg>) {
        while let Some(msg) = rx.recv().await {
            match msg {
                Msg::Records(records) => {
                    for record in &records {
                        if let Err(e) = Self::write_record(&pool, record).await {
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

// ── Insertion ───────────────────────────────────────────────────────────────

impl PostgresWriter {
    async fn write_record(pool: &PgPool, record: &ResolvedRecord) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        Self::insert_record(record, &mut tx).await?;

        for metric in &record.metrics {
            Self::insert_metric(metric, &mut tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

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
