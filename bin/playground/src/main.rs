use domain::{
    asset::AssetId,
    order::{LimitOrder, OrderId, OrderSide, Price, Quantity, Timestamp},
};
use orderbook::lob::l3::{Book, BookConfig, SideConfig};

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

        let mut book = set_up_boook();
        let order = LimitOrder::new(
            OrderId::new(0),
            Timestamp::new(0),
            Quantity::new(10),
            OrderSide::Bid,
            Price::new(100),
        );
        book.add_order(order);
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

// ── Orderbook utilities ──────────────────────────────────────────────────────

fn set_up_boook() -> Book {
    // 1. Configure preallocation capacities
    const LOOKUP_CAPACITY: usize = 10_000;
    const SIDE_CAPACITY: usize = 1_000;
    const LEVEL_CAPACITY: usize = 100;

    let book_config = BookConfig::new(LOOKUP_CAPACITY, SIDE_CAPACITY, SIDE_CAPACITY);

    let bid_side_config = SideConfig::new(LEVEL_CAPACITY);
    let ask_side_config = SideConfig::new(LEVEL_CAPACITY);

    // 2. Create the order book
    let asset_id = AssetId::new(0);

    Book::new(asset_id, book_config, bid_side_config, ask_side_config)
}
