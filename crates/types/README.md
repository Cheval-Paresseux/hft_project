# Types

Shared domain definitions for the workspace, providing a common vocabulary and set of abstractions for representing trading activity. The crate aims to keep domain concepts consistent across market data, trading, simulation, and execution components.

## Overview

The `types` crate establishes the shared language of the workspace. It defines the fundamental concepts used to represent trading activity, independently of the components that consume or implement them.

Its scope is centred on two areas:

- **Primitive domain types:** Strongly typed representations of values such as prices, quantities, timestamps, and identifiers, alongside common enumerations.
- **Trading abstractions:** Shared interfaces and representations for orders, allowing different components to interact through consistent domain definitions.

The objective is to avoid duplicated definitions and incompatible representations across the workspace, while keeping the crate independent of higher-level trading logic.

## Example

```rust
use types::order::{
    LimitOrder, Order, OrderId, OrderSide, Price, Quantity,
    RestingLimitOrder, TimeInForce, Timestamp,
};

let mut order = LimitOrder::new(
    OrderId::new(1),
    Quantity::new(10),
    Timestamp::new(1_000),
    OrderSide::Bid,
    Price::new(100),
    TimeInForce::GoodTillCanceled,
);

// Reduce the order to the quantity left after a partial execution.
assert_eq!(order.reduce(Quantity::new(6)), Ok(()));
assert_eq!(order.quantity(), Quantity::new(6));

// Only a larger quantity is accepted afterwards.
assert_eq!(order.increase(Quantity::new(8)), Ok(()));
assert_eq!(order.quantity(), Quantity::new(8));

// Convert it to the representation retained by an order book.
let resting = RestingLimitOrder::from(order);
assert_eq!(resting.quantity(), Quantity::new(8));
```

## Architecture

The crate follows a principle of **cohesion and separation of concerns**. Domain concepts should be grouped according to their relationships and responsibilities, avoiding both unnecessary fragmentation and overly broad abstractions.

- **Separate independent concepts:** Types with distinct responsibilities, such as timestamps and orders, should remain independent.
- **Group related concepts:** Closely related types, such as market, limit, stop, and stop-limit orders, should belong to a common conceptual domain.
- **Avoid unnecessary coupling:** Shared definitions should remain reusable without introducing dependencies on the higher-level components that consume them.

The objective is to maintain a coherent domain model that can evolve alongside the rest of the workspace.

## Crate Structure

For the current needs, the crate is organised into two components:

```text
types
└── order
    ├── fields
    │   ├── id.rs        # OrderId and packed construction
    │   ├── price.rs     # Price
    │   ├── quantity.rs  # Quantity
    │   ├── side.rs      # OrderSide
    │   ├── tif.rs       # TimeInForce / MarketTimeInForce
    │   └── timestamp.rs # Timestamp
    │
    └── orders
        ├── common.rs   # Order / IncomingOrder, OrderError
        ├── base        # Limit, Market, Stop, Stop-Limit
        └── resting     # RestingLimitOrder
```

## Development Roadmap

> *Grows with the development of other parts when necessary*
