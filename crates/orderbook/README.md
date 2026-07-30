# Order Book

This module provides a collection of order book implementations, each optimized for a different balance between update latency, query performance, memory footprint, and maintained state. Rather than promoting a single "best" implementation, the goal is to offer well-engineered designs suited to different workloads.

## Design

There is no universally optimal order book implementation. Every design optimizes for specific access patterns while accepting trade-offs elsewhere. This module focuses on practical implementations with clear performance characteristics rather than attempting to maximize every metric simultaneously.

The order book is intentionally decoupled from the matching engine. The matching engine is responsible for orchestrating order flow and executing matching logic, while the order book is responsible solely for maintaining market state.

## Motivation

> **"Why maintain an order book when many providers already expose Level 2 market data?"**

Level 3 market data is the most granular representation of an exchange's order flow. It provides the highest-fidelity view of the market, the lowest latency for state reconstruction, and complete control over how market data is aggregated and consumed. Relying solely on Level 2 data delegates these decisions to a third party.

- **Control the trade-offs.** Reconstruct the market state directly from exchange events and choose your own balance between latency, memory usage, and data quality.
- **Greater flexibility.** Compute custom market microstructure metrics and retain metadata that aggregated feeds typically discard.
- **Core infrastructure.** The order book is the central data structure of any market data engine and can be adapted to the needs of trading, simulation, or research systems.

## Implementations

### Limit Order Book (Level 3)

**Module**

```text
orderbook::lob::l3::*
```

This implementation maintains a Level 3 limit order book containing only limit orders. The order book is intentionally agnostic to order-entry policies. Market orders, time-in-force instructions (IOC, FOK, GTC, etc.), self-trade prevention, and other execution rules are expected to be handled by the matching engine. The order book only maintains the resting state of the market.

#### Design

This implementation prioritizes simplicity while incorporating practical optimizations. It is organized into three components:

- **Level** — stores resting orders at a single price level while preserving price-time priority.
- **Side** — manages the ordered collection of price levels and delegates operations to the appropriate level.
- **Book** — exposes the public API and maintains an order lookup table for efficient cancel and modify operations.

The book is configured through a `BookConfig` and one `SideConfig` per side. These currently define simple preallocation capacities but are designed to support more advanced allocation strategies in the future (for example, allocating more capacity near the top of the book).

To minimize memory usage, resting orders are represented by `LimitRestingOrder`, which stores only the fields required by the order book:

```rust
id: u64,
timestamp: u64,
quantity: u64,
```

for a total size of only 24 bytes.

#### Usage

```rust
// 1. Configure preallocation capacities
const LOOKUP_CAPACITY: usize = 10_000;
const SIDE_CAPACITY: usize = 1_000;
const LEVEL_CAPACITY: usize = 100;

let book_config = BookConfig::new(
    LOOKUP_CAPACITY,
    SIDE_CAPACITY,
    SIDE_CAPACITY,
);

let bid_side_config = SideConfig::new(LEVEL_CAPACITY);
let ask_side_config = SideConfig::new(LEVEL_CAPACITY);

// 2. Create the order book
let asset_id = AssetId::new(0);

let mut book = Book::new(
    asset_id,
    book_config,
    bid_side_config,
    ask_side_config,
);

// Add an order
let order = LimitOrder::new(
    OrderId::new(0),
    Timestamp::new(0),
    Quantity::new(10),
    OrderSide::Bid,
    Price::new(100),
);

book.add_order(order);

// Modify an order
let modification = Modification::new_reduce_quantity(
    ModificationId::new(1),
    Timestamp::new(1),
    OrderId::new(0),
    Quantity::new(5),
);

let modify_result = book.modify_order(modification);

// Cancel an order
let cancel_result = book.cancel_order(OrderId::new(0));

// Fill resting liquidity
let fill_result = book.fill(
    Quantity::new(5),
    Price::new(100),
    OrderSide::Bid,
);
```