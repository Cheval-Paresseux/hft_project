# Orderbook

Order book data structures for maintaining resting orders at different levels of granularity.

## Overview

An order book is a data structure responsible for organising and maintaining resting orders. Orders are typically arranged by price and time priority, allowing the system to efficiently identify the best available prices and access orders in their execution sequence. It must support the fundamental operations required to manage this state, including insertion, modification, cancellation, and retrieval.

The `orderbook` crate focuses on **order storage and management, not matching logic**. It provides the data structures and operations required by a matching engine to inspect and modify resting orders. This includes exposing the relevant book state, maintaining ordering and consistency, and supporting the mutations required during order processing.

Matching rules, execution decisions, and trade generation belong to the matching engine. This separation keeps the order book focused on maintaining an accurate representation of resting liquidity, while allowing matching logic to evolve independently of the underlying data structures.

## Example

```rust
use book::{
    OrderMutationsPolicy,
    level3::{L3OrderBook, VecL3OrderBook},
    trigger::{TriggerBook, TriggerCondition, VecTriggerBook},
};

use types::order::{
    LimitOrder, Order, OrderId, OrderSide, Price, Quantity, RestingLimitOrder, StopLimitOrder,
    TimeInForce, Timestamp,
};

let mut book = VecL3OrderBook::<LimitOrder, RestingLimitOrder>::new(
    OrderMutationsPolicy::KeepPriority,
    OrderMutationsPolicy::KeepPriority,
);

book.insert(LimitOrder::new(
    OrderId::new(1),
    Quantity::new(10),
    Timestamp::new(1_000),
    OrderSide::Bid,
    Price::new(100),
    TimeInForce::GoodTillCanceled,
))
.unwrap();

// Best order on the bid side.
let (side, price, order) = book.best_order(OrderSide::Bid).unwrap();
assert_eq!(
    (side, price, order.quantity()),
    (OrderSide::Bid, Price::new(100), Quantity::new(10))
);

// Reduce the order to its remaining quantity, then remove it.
book.reduce(OrderId::new(1), Quantity::new(5)).unwrap();
assert_eq!(
    book.quantity_at(OrderSide::Bid, Price::new(100)),
    Quantity::new(5)
);

book.remove(OrderId::new(1)).unwrap();
assert!(book.best_order(OrderSide::Bid).is_none());

// Trigger book releases orders once the observed price crosses their threshold.
let mut triggers = VecTriggerBook::<Price, StopLimitOrder>::new(
    TriggerCondition::AtOrAbove,
    OrderMutationsPolicy::KeepPriority,
    OrderMutationsPolicy::KeepPriority,
);

triggers
    .insert(
        Price::new(105),
        StopLimitOrder::new(
            OrderId::new(2),
            Quantity::new(5),
            Timestamp::new(1_000),
            OrderSide::Bid,
            Price::new(105),
            Price::new(106),
            TimeInForce::GoodTillCanceled,
        ),
    )
    .unwrap();

assert!(triggers.trigger(Price::new(104)).is_empty());
let triggered = triggers.trigger(Price::new(105));
assert_eq!(triggered.len(), 1);
```

## Architecture

The complexity of order-book operations and their demanding performance requirements make it difficult to define a universally optimal data structure. Different workloads and access patterns favour different trade-offs in memory usage, operation complexity, cache locality, and throughput.

The crate is therefore designed around **book abstractions** and **concrete implementations**, keeping the expected behaviour of a book separate from the data structures used to provide it.

- **Distinct book models:** Different levels of market information and different operational requirements call for different abstractions. Level 2 books organise aggregated quantities at price levels, Level 3 books retain individual orders and their priority, while trigger books manage conditional orders independently of the regular price-time structure.
- **Implementation independence:** Each abstraction defines the operations and guarantees expected by its consumers without prescribing a particular storage mechanism. Multiple implementations can therefore provide the same interface while making different performance trade-offs.
- **Performance-oriented design:** Data structures should be selected according to their operational characteristics, including insertion, cancellation, modification, price-level lookup, and traversal. No implementation is assumed to be optimal for every workload.
- **Separation of responsibilities:** The order book maintains and exposes its state, while matching logic remains the responsibility of the matching engine. This allows both components to evolve independently.

The objective is to provide a consistent set of interfaces while leaving room for specialised implementations optimised for different execution environments and performance requirements.

## Crate Structure

```text
orderbook
├── common         # BookError, OrderMutationsPolicy
├── level3         # order-granular book interface
│   ├── common     # L3OrderBook, checked quantity arithmetic
│   └── vec        # VecL3OrderBook: sorted levels, FIFO orders, order index
|
└── trigger        # threshold-triggered orders
    ├── common     # TriggerBook, TriggerCondition
    └── vec        # VecTriggerBook: threshold-sorted orders
```

## Development Roadmap

- [x] **L3 order book:** Trait and `Vec`-backed implementation.
- [x] **Trigger book:** Trait, conditions, and `Vec`-backed implementation.
- [ ] **Level 2 order book:** Price-level aggregation behind a separate trait.
- [ ] **Alternative implementations:** Tree- or heap-backed books for different access patterns.
- [ ] **Benchmarking:** Compare implementations and guide future optimisation.
