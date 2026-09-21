# Orderbook

A collection of **Level 3 (L3)** and **Level 2 (L2)** order book structures, exposed through a common interface.

The crate separates **what an order book does** from **how it is implemented**:

- Stable `L3OrderBook` and `L2OrderBook` traits define the operations supported by each level of granularity.
- Concrete implementations provide different data structures and optimization trade-offs.

This allows each strategy or matching engine to choose the implementation that best fits its requirements, without coupling consuming code to a particular data structure.

## Motivation

### Why implement order book structures?

Maintaining order book data is fundamental to the rest of the system. It allows raw order feeds to be maintained in a structured form and enables downstream components to perform only the computations they require, reducing unnecessary work and improving the efficiency of data processing.

An **L3 order book** maintains every individual resting order rather than aggregating orders at each price level. This preserves order-level information, making it possible to:

- inspect the best individual order;
- target and modify a specific order by its ID;
- process orders according to price-time priority;
- reconstruct the order-level state represented by the feed.

An **L2 order book** aggregates orders by price level. This sacrifices individual order information in exchange for a more compact representation focused on aggregate quantities and prices.

The two representations therefore serve different purposes: L3 provides greater granularity and information, while L2 provides a representation optimized for price-level queries and aggregate market depth.

## Roadmap

The `vec` implementation is the first of what is intended to be a family of implementations, each tuned for a specific access pattern — e.g. a tree- or heap-backed book for insertion-heavy feeds, or an array/map-backed book for tight quoting loops. All of them will stay behind the `L3OrderBook` interface.

There is no Level-2 implementations yet, but it is planned for later.

## Crate Architecture

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
└── level2
    ├── traits.rs 
    └── ...
```

## Data Model

- `OrderId`, `Price`, `Quantity`, `Timestamp`, `OrderSide` — primitive newtype fields, so quantities and prices cannot be mixed accidentally.
- `LimitOrder` — an incoming order, carrying `id`, `timestamp`, `quantity`, `side` and `price`.

- L3 only: `RestingOrder` — an order stored in the book (id, timestamp, quantity); a `LimitOrder` is reduced to a `RestingOrder` on arrival.

Operations that cannot be performed are reported through `OrderBookError`:

- `OrderIdNotFound` — the order is not in the book;
- `PriceLevelNotFound` — there is no level at that price;
- `FillExceedsOrderQuantity` — a fill is larger than the resting quantity.

## **Level 3 (L3) Order Book**

### An interface, many implementations

The crate/level3 is organized around the `L3OrderBook` trait:

```rust
pub trait L3OrderBook {
    fn add_order(&mut self, order: LimitOrder);
    fn cancel_order(&mut self, order_id: OrderId) -> Result<(), OrderBookError>;
    fn modify_order(&mut self, order_id: OrderId, new_quantity: Quantity) -> Result<(), OrderBookError>;
    fn fill_order(&mut self, order_id: OrderId, fill_quantity: Quantity) -> Result<(), OrderBookError>;

    fn order(&self, order_id: OrderId) -> Result<(OrderSide, Price, Quantity), OrderBookError>;
    fn top_order(&self, side: OrderSide) -> Option<(OrderId, Price, Quantity)>;
    fn orders_at(&self, side: OrderSide, price: Price) -> Result<Vec<(OrderId, Quantity)>, OrderBookError>;
    fn orders_up_to(&self, side: OrderSide, bound: Option<Price>) -> Vec<(OrderId, Price, Quantity)>;

    fn top_price(&self, side: OrderSide) -> Option<Price>;
    fn quantity_at(&self, side: OrderSide, price: Price) -> Quantity;
    fn quantity_up_to(&self, side: OrderSide, bound: Option<Price>) -> Quantity;
}
```

The first group mutates the book; the second group queries it.

Concrete implementations are provided under `level3::`, e.g. `VecL3OrderBook`. Nothing in the trait leaks implementation details, so a strategy can be written against the trait and later swapped to a different implementation with no changes to its logic.

### The `vec` implementation

`VecL3OrderBook` is a straightforward, cache-friendly implementation built on `Vec`s:

- each **side** keeps its levels sorted — asks ascending, bids descending — found by binary search (`partition_point`);
- each **level** keeps its orders in arrival order, so the first order is the FIFO-best one;
- each **level** caches its `total_quantity`, making `quantity_at` O(1) once the level is located;
- the **book** keeps a `HashMap<OrderId, (OrderSide, Price)>` index so an order can be routed to its side and level in O(1).

The result is a simple, predictable structure with good constant factors for add/cancel/modify and fast best-order access.

### Usage

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

// The best order and the best price on each side.
assert_eq!(book.top_order(OrderSide::Bid), Some((1.into(), 100.into(), 10.into())));
assert_eq!(book.top_price(OrderSide::Bid), Some(100.into()));
assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), 10.into());

// The order is partially filled...
book.fill_order(1.into(), 4.into()).unwrap();

// ...then modified, and finally cancelled.
book.modify_order(1.into(), 5.into()).unwrap();
book.cancel_order(1.into()).unwrap();

assert_eq!(book.top_order(OrderSide::Bid), None);
```

Consumer code only ever depends on the `L3OrderBook` trait, so the concrete book can be swapped without touching the strategy.
