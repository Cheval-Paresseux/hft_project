use telemetry::tracing::{
    capture::Scope,
    enrichment::Registry,
    handoff::{Channel, Subscriber},
    writers::{FileWriter, StdoutWriter},
};
use telemetry::{
    schema::{metrics, scopes},
    tracing::writers::PostgresWriter,
};

// ── Main ─────────────────────────────────────────────────────────────────────


#[tokio::main]
async fn main() {
    let registry = Registry::default();
    let channel = Channel::unbounded();

    let mut subscriber = Subscriber::new(channel.receiver, &registry);
    subscriber.add_writer(StdoutWriter);

    let file_sink = create_file_sink();
    subscriber.add_writer(file_sink);

    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let (writer, guard) = PostgresWriter::new(&database_url).await;
    subscriber.add_writer(writer);

    // -----

    let mut scope = Scope::init(scopes::INITIALIZATION, channel.sender, None);
    scope.process_memory_snapshot();

    {
        let _ = scope.child(scopes::SHUTDOWN, None);
    }

    // ---

    scope.metric(metrics::END_TIME, 0, None);
    drop(scope);

    subscriber.drain();
    guard.shutdown().await;
}

// ── Telemetry utilities ──────────────────────────────────────────────────────

pub fn create_file_sink() -> FileWriter {
    let path = "/home/mathis/Documents/code/hft/bin/market_simulator/src/test.txt";
    FileWriter::existing(path).expect("failed")
}
