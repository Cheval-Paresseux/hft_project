use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use domain::asset::AssetId;
use domain::order::{LimitOrder, Modification, ModificationId, OrderId, OrderSide, Price, Quantity, Timestamp};
use orderbook::lob::l3::{Book, BookConfig, SideConfig};

// ── Helpers ───────────────────────────────────────────────────────────────────

const LOOKUP_CAPACITY: usize = 10_000;
const SIDE_CAPACITY: usize = 1_000;
const LEVEL_CAPACITY: usize = 100;
const DEFAULT_PRICE: u64 = 100;

fn create_book() -> Book {
    Book::new(
        AssetId::new(1),
        BookConfig::new(LOOKUP_CAPACITY, SIDE_CAPACITY, SIDE_CAPACITY),
        SideConfig::new(LEVEL_CAPACITY),
        SideConfig::new(LEVEL_CAPACITY),
    )
}

fn create_order(id: u64, price: u64) -> LimitOrder {
    LimitOrder::new(
        OrderId::new(id),
        Timestamp::new(id),
        Quantity::new(10),
        OrderSide::Bid,
        Price::new(price),
    )
}

fn create_modification_reduce_quantity(timestamp: u64, order_id: u64) -> Modification {
    Modification::new_reduce_quantity(
        ModificationId::new(1),
        Timestamp::new(timestamp),
        OrderId::new(order_id),
        Quantity::new(5),
    )
}

fn create_modification_increase_quantity(timestamp: u64, order_id: u64) -> Modification {
    Modification::new_increase_quantity(
        ModificationId::new(1),
        Timestamp::new(timestamp),
        OrderId::new(order_id),
        Quantity::new(15),
    )
}

// ── Setups ────────────────────────────────────────────────────────────────────

fn setup_book_one_level(n_orders: usize) -> Book {
    let mut book = create_book();

    for i in 0..n_orders as u64 {
        let order = create_order(i, DEFAULT_PRICE);
        book.add_order(order);
    }

    book
}

fn setup_book_ten_level(n_orders: usize) -> Book {
    let mut book = create_book();

    let mut id = 0;
    for _ in 0..n_orders as u64 {
        for j in 0..10 {
            let order = create_order(id, DEFAULT_PRICE + 2*j);
            book.add_order(order);

            id += 1;
        }
    }

    book
}

// ── Actions ───────────────────────────────────────────────────────────────────

fn add_one_order(book: &mut Book, price: u64) {
    let order = create_order(100_000, price);
    std::hint::black_box(book.add_order(order));
}

fn cancel_one_order(book: &mut Book, order_id: OrderId) {
    let _ = std::hint::black_box(book.cancel_order(order_id));
}

fn modify_reduce_one_order(book: &mut Book, timestamp: u64, order_id: u64) {
    let modification = create_modification_reduce_quantity(timestamp, order_id);
    let _ = std::hint::black_box(book.modify_order(modification));
}

fn modify_increase_one_order(book: &mut Book, timestamp: u64, order_id: u64) {
    let modification = create_modification_increase_quantity(timestamp, order_id);
    let _ = std::hint::black_box(book.modify_order(modification));
}

// ── Bench ─────────────────────────────────────────────────────────────────────

fn bench_add_order(c: &mut Criterion) {
    c.bench_function("add_at_existing_level_in_capacity", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_one_level(LEVEL_CAPACITY - 50)),
            |mut book| add_one_order(&mut book, DEFAULT_PRICE),
            BatchSize::PerIteration,
        );
    });

    c.bench_function("add_at_existing_level_exceeds_capacity", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_one_level(LEVEL_CAPACITY)),
            |mut book| add_one_order(&mut book, DEFAULT_PRICE),
            BatchSize::PerIteration,
        );
    });

    c.bench_function("add_at_empty_worst_with_one_level", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_one_level(LEVEL_CAPACITY)),
            |mut book| add_one_order(&mut book, DEFAULT_PRICE - 5),
            BatchSize::PerIteration,
        );
    });

    c.bench_function("add_at_empty_worst_with_ten_level", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_ten_level(LEVEL_CAPACITY)),
            |mut book| add_one_order(&mut book, DEFAULT_PRICE - 5),
            BatchSize::PerIteration,
        );
    });

    c.bench_function("add_at_empty_best_with_one_level", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_one_level(LEVEL_CAPACITY)),
            |mut book| add_one_order(&mut book, DEFAULT_PRICE + 5),
            BatchSize::PerIteration,
        );
    });

    c.bench_function("add_at_empty_best_with_ten_levels", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_ten_level(LEVEL_CAPACITY)),
            |mut book| add_one_order(&mut book, DEFAULT_PRICE + 50),
            BatchSize::PerIteration,
        );
    });

    c.bench_function("add_at_empty_middle_with_ten_levels", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_ten_level(LEVEL_CAPACITY)),
            |mut book| add_one_order(&mut book, DEFAULT_PRICE + 10),
            BatchSize::PerIteration,
        );
    });
}

fn bench_cancel_order(c: &mut Criterion) {
    c.bench_function("cancel_first_order", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_one_level(LEVEL_CAPACITY)),
            |mut book| cancel_one_order(&mut book, OrderId::new(0)),
            BatchSize::PerIteration,
        );
    });

    c.bench_function("cancel_last_order", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_one_level(LEVEL_CAPACITY)),
            |mut book| cancel_one_order(&mut book, OrderId::new(LEVEL_CAPACITY as u64)),
            BatchSize::PerIteration,
        );
    });

    c.bench_function("cancel_last_order_overpopulated", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_one_level(LEVEL_CAPACITY * 10)),
            |mut book| cancel_one_order(&mut book, OrderId::new(LEVEL_CAPACITY as u64 * 10)),
            BatchSize::PerIteration,
        );
    });
}

fn bench_modify_order(c: &mut Criterion) {
    c.bench_function("modify_reduce_first_order", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_one_level(LEVEL_CAPACITY)),
            |mut book| modify_reduce_one_order(&mut book, 10_000, 0),
            BatchSize::PerIteration,
        );
    });

    c.bench_function("modify_reduce_last_order", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_one_level(LEVEL_CAPACITY)),
            |mut book| modify_reduce_one_order(&mut book, 10_000, LEVEL_CAPACITY as u64),
            BatchSize::PerIteration,
        );
    });

    c.bench_function("modify_increase_first_order", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_one_level(LEVEL_CAPACITY)),
            |mut book| modify_increase_one_order(&mut book, 10_000, 0),
            BatchSize::PerIteration,
        );
    });

    c.bench_function("modify_increase_last_order", |b| {
        b.iter_batched(
            || std::hint::black_box(setup_book_one_level(LEVEL_CAPACITY)),
            |mut book| modify_increase_one_order(&mut book, 10_000, LEVEL_CAPACITY as u64),
            BatchSize::PerIteration,
        );
    });
}

criterion_group!(benches, bench_add_order, bench_cancel_order, bench_modify_order);
criterion_main!(benches);
