# Orderbook

A collection of **level-3 (L3) order book** structures, exposed behind a single common interface.

The crate separates **what an order book does** from **how it does it**:

- a stable `L3OrderBook` trait defines the operations any order book must support;
- concrete implementations live behind that trait, and each one makes different trade-offs.

This lets every **strategy or matching engine pick the implementation that best fits its own optimization requirements**, without the consuming code depending on any particular data structure.

## Motivation

> **Why implement order book structures?**

Being able to maintain order book data is key to the rest of the system. It allows to take the raw feed of orders directly and then perform only the required amount of computation, hence maximizing the efficiency and the quality of the data you ingest.

An L3 order book does not aggregate: it keeps every individual resting order, which is what makes it possible to query the *best* order, to target a specific order, and to reconstruct the feed faithfully.

## Design

### An interface, many implementations

The whole crate is organized around the `L3OrderBook` trait:

```rust
pub trait L3OrderBook {
    fn add_order(&mut self, order: LimitOrder);
    fn cancel_order(&mut self, order_id: OrderId) -> Result<(), OrderBookError>;
    fn modify_order(&mut self, order_id: OrderId, new_quantity: Quantity) -> Result<(), OrderBookError>;
    fn fill(&mut self, order_id: OrderId, quantity: Quantity) -> Result<(), OrderBookError>;

    fn best(&self, side: OrderSide) -> Option<(OrderId, Price, Quantity)>;
    fn best_price(&self, side: OrderSide) -> Option<Price>;
    fn quantity_at(&self, side: OrderSide, price: Price) -> Quantity;
}
```

The first group mutates the book; the second group queries it.

Concrete implementations are provided under `level3::`, e.g. `VecL3OrderBook`. Nothing in the trait leaks implementation details, so a strategy can be written against the trait and later swapped to a different implementation with no changes to its logic.

### Choosing an implementation

Different trading strategies access the book in different ways, and the "best" data structure depends on which operations dominate:

- **insert-heavy flows** (market-data ingestion) benefit from cheap `add_order`;
- **aggressive flows** (market-making, quoting) spend most of their time on `cancel` / `modify`;
- **latency-sensitive quotes** need `best` / `best_price` to be as close to O(1) as possible;
- **aggregation and risk checks** rely on `quantity_at`.

Because these requirements can conflict, the crate deliberately does **not** hard-code a single structure: each implementation is a point in that trade-off space, and the matching engine or strategy selects the one that fits its workload.

### Responsibility of the order book

The order book is responsible for holding the orders data while ensuring FIFO. This separation of concerns is explicit: the order book handles the *structure*, the matching engine handles the *matching*.

It is important to notice that the FIFO maintained by the order book only concerns the orders themselves, **not their metadata**. Re-ordering due to timestamp, or an increase in quantity, is managed by the matching engine.

## Data Model

- `OrderId`, `Price`, `Quantity`, `Timestamp`, `OrderSide` — primitive newtype fields, so quantities and prices cannot be mixed accidentally.
- `LimitOrder` — an incoming order, carrying `id`, `timestamp`, `quantity`, `side` and `price`.
- `RestingOrder` — an order stored in the book (id, timestamp, quantity); a `LimitOrder` is reduced to a `RestingOrder` on arrival.

Operations that cannot be performed are reported through `OrderBookError`:

- `OrderIdNotFound` — the order is not in the book;
- `PriceLevelNotFound` — there is no level at that price;
- `FillExceedsOrderQuantity` — a fill is larger than the resting quantity.

## Architecture

```text
orderbook
├── order            # domain primitives (fields + LimitOrder / RestingOrder)
├── errors           # OrderBookError
└── level3
    ├── traits.rs    # L3OrderBook interface
    └── vec          # VecL3OrderBook implementation
        ├── book.rs  # order book: two sides + order index
        ├── side.rs  # one side: sorted levels
        └── level.rs # one level: FIFO orders + total quantity
```

### The `vec` implementation

`VecL3OrderBook` is a straightforward, cache-friendly implementation built on `Vec`s:

- each **side** keeps its levels sorted — asks ascending, bids descending — found by binary search (`partition_point`);
- each **level** keeps its orders in arrival order, so the first order is the FIFO-best one;
- each **level** caches its `total_quantity`, making `quantity_at` O(1) once the level is located;
- the **book** keeps a `HashMap<OrderId, (OrderSide, Price)>` index so an order can be routed to its side and level in O(1).

The result is a simple, predictable structure with good constant factors for add/cancel/modify and fast best-order access.

## Usage

```rust
use orderbook::{
    level3::{L3OrderBook, VecL3OrderBook},
    order::{LimitOrder, OrderSide},
};

let mut book = VecL3OrderBook::default();

// Ingest an incoming limit order.
book.add_order(LimitOrder::new(
    1.into(),              // order id
    0.into(),              // timestamp
    10.into(),             // quantity
    OrderSide::Bid,        // side
    100.into(),            // limit price
));

// The best bid is now this order.
assert_eq!(book.best(OrderSide::Bid), Some((1.into(), 100.into(), 10.into())));
assert_eq!(book.best_price(OrderSide::Bid), Some(100.into()));
assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), 10.into());

// The order is partially filled...
book.fill(1.into(), 4.into()).unwrap();

// ...then modified, and finally cancelled.
book.modify_order(1.into(), 5.into()).unwrap();
book.cancel_order(1.into()).unwrap();

assert_eq!(book.best(OrderSide::Bid), None);
```

Consumer code only ever depends on the `L3OrderBook` trait, so the concrete book can be swapped without touching the strategy.

## Roadmap

The `vec` implementation is the first of what is intended to be a family of implementations, each tuned for a specific access pattern — e.g. a tree- or heap-backed book for insertion-heavy feeds, or an array/map-backed book for tight quoting loops. All of them will stay behind the `L3OrderBook` interface.
