# Telemetry

A structured observability framework for latency-critical systems, keeping instrumentation predictable, lightweight, and explicitly controlled.

## Overview

The `telemetry` crate records structured execution data with a **Scope–Metric** model, conceptually similar to the Span–Event model of [`tracing`](https://docs.rs/tracing) but deliberately constrained around memory usage and hot-path overhead.

Its central design principle is:

> **The hot path records only what is strictly necessary. Everything else belongs to the cold path.**

Records are fixed-size, handed off explicitly, and processed later. This makes the cost of instrumentation easier to reason about in performance-sensitive systems such as high-frequency trading, market-data processing, and low-latency services.

## Example

```rust
use telemetry::tracing::capture::{MetricId, Scope, ScopeId};
use telemetry::tracing::enrichment::{MetricMetadata, Registry, ScopeMetadata};
use telemetry::tracing::handoff::{Channel, Subscriber};
use telemetry::tracing::writers::StdoutWriter;

const ORDER_PROCESSING: ScopeId = ScopeId::new(0x0100, 1);
const VALIDATION: ScopeId = ScopeId::new(0x0100, 2);
const VALIDATION_LATENCY: MetricId = MetricId::new(0x0100, 1);

fn main() {
    // 1. Register the metadata used to resolve IDs on the cold path.
    let mut registry = Registry::new();

    registry.build_scopes(&[
        ScopeMetadata {
            id: ORDER_PROCESSING,
            name: "order_processing",
            description: "Processing of one incoming order.",
            payload_info: "order id",
        },
        ScopeMetadata {
            id: VALIDATION,
            name: "validation",
            description: "Order validation step.",
            payload_info: "N/A",
        },
    ]);

    registry.build_metrics(&[MetricMetadata {
        id: VALIDATION_LATENCY,
        name: "validation_latency",
        unit: "ns",
        description: "Time spent validating the order.",
        payload_info: "N/A",
    }]);

    // 2. Create the handoff channel and a subscriber with a writer.
    let channel = Channel::unbounded();

    let mut subscriber = Subscriber::new(channel.receiver, &registry);
    subscriber.add_writer(StdoutWriter);

    // 3. Instrument the hot path with nested scopes.
    {
        let order = Scope::init(ORDER_PROCESSING, channel.sender, Some(42));

        {
            let mut validation = order.child(VALIDATION, None);
            validation.metric(VALIDATION_LATENCY, 4_200, None);
        }
    }

    // 4. Drain the channel and let the cold path resolve and write records.
    subscriber.drain();
}
```

## Architecture

Telemetry separates **capture** from **interpretation and persistence**. The boundary between the two is explicit.

```text
                    HOT PATH
                        │
                        │ Scope / Metric
                        ▼
                ┌───────────────┐
                │    Record     │
                └───────┬───────┘
                        │ handoff
                        ▼
                    Channel
                        │
                        ▼
                    COLD PATH
                        │
             ┌──────────┼──────────┐
             ▼          ▼          ▼
          Resolve    Dispatch    Write
          Metadata   Records     / Persist
```

A **Scope** defines a meaningful, nestable execution context; a **Metric** records one piece of information within that context. Keeping context on the scope lets metrics stay small and avoids repeating identifiers in every entry.

```text
OrderProcessing
├── RECEIVED
├── VALIDATION_DURATION
├── RISK_CHECK_DURATION
└── SENT
```

Nested scopes make the captured records form a graph of related execution contexts. After handoff, a `Channel` carries records to a `Subscriber`, which resolves IDs through a `Registry` and dispatches the resulting `ResolvedRecord`s to one or more writers.

## Crate Structure

```text
telemetry
├── collectors     # process, allocator, and clock measurements
├── schema         # standard scope and metric definitions
└── tracing
    ├── capture    # scopes, metrics, and fixed-size records
    ├── enrichment # metadata resolution and stable instance UUIDs
    ├── handoff    # channel transport and subscriber dispatch
    └── writers    # stdout, file, memory, and PostgreSQL output
```

## Development Roadmap

- [x] **Core pipeline:** Capture, enrichment, handoff, and persistence.
- [x] **Output backends:** Stdout, file, memory, and PostgreSQL writers.
- [ ] **Benchmarking:** Measure hot-path overhead and cold-path throughput.
- [ ] **Instrumentation macros:** Improve ergonomics without sacrificing explicitness.
- [ ] **Performance R&D:** Investigate further optimisations guided by benchmark results.
