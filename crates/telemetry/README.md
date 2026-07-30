# Telemetry

A structured observability framework designed for **latency-critical systems**, where instrumentation must remain predictable, lightweight, and under explicit control.

Telemetry uses a **Scope–Metric architecture**, conceptually similar to the Span–Event model of [`tracing`](https://docs.rs/tracing), but deliberately constrained around memory usage and hot-path overhead.

The central design principle is simple:

> **The hot path records only what is strictly necessary. Everything else belongs to the cold path.**

This makes Telemetry particularly suited to systems such as **high-frequency trading, market-data processing, low-latency services, and performance-sensitive infrastructure**.

## Motivation

> **Why build a custom telemetry framework instead of using `tracing`, OpenTelemetry, or conventional metrics libraries?**

General-purpose observability frameworks provide extensive functionality and flexibility. That flexibility comes with trade-offs that are not always desirable on latency-critical paths.

Telemetry instead makes a different trade:

- **predictability over flexibility**
- **explicit memory usage over dynamic allocation**
- **structured records over free-form logs**
- **batching and deferred processing over immediate output**
- **minimal hot-path work over feature-rich instrumentation**

The hot path should perform only the operations required to capture the telemetry data. Resolution, formatting, routing, persistence, and other expensive operations are delegated to the cold path.

In particular, instrumentation is designed around:

- no heap allocation for the core record;
- no system calls from the recording path;
- fixed-size, cache-friendly records;
- explicit handoff from the hot path to the cold path;
- flushing only when explicitly requested;
- deferred processing and writing.

The goal is not to make telemetry invisible. It is to make its **cost explicit, bounded, and understandable**.

---

## Design

### Scope–Metric architecture

Telemetry separates what would traditionally be a single log entry into two distinct concepts:

- **Scope** — provides the context in which something happened;
- **Metric** — records a specific piece of information within that context.

A `Scope` therefore represents a meaningful unit of execution, while `Metric`s describe what happened inside it.

```text
Scope
├── Metric
├── Metric
├── Metric
└── ...
```

Scopes can be nested:

```text
Initialization
├── DatabaseConnection
│   ├── ConnectionAttempt
│   └── ConnectionEstablished
└── ConfigurationLoading
    ├── FileRead
    └── Parsing
```

This gives telemetry data an inherent hierarchical structure without requiring every metric to carry its complete context.

### Why scopes?

A Scope should have both **functional and technical meaning**.

For example, instead of recording dozens of unrelated messages:

```text
"starting order processing"
"received market data"
"order validation took 12µs"
"risk check took 4µs"
"order sent"
```

the instrumentation can define a meaningful execution context:

```text
OrderProcessing
├── RECEIVED
├── VALIDATION_DURATION
├── RISK_CHECK_DURATION
└── SENT
```

The Scope provides the context once. Metrics only need to carry the information specific to the event being recorded.

This has two important consequences:

1. **Metrics can remain extremely small.**
2. **The resulting data naturally forms a graph of related execution contexts.**

---

## Deliberate constraints

Telemetry intentionally provides fewer abstractions than general-purpose logging frameworks.

This is a feature, not a limitation.

Instrumentation forces the developer to answer questions such as:

- What actually constitutes a meaningful Scope?
- Which metrics are worth recording?
- Which information belongs in the Scope?
- Which information should be attached to an individual Metric?
- Is this event important enough to record at all?

These constraints are intended to prevent telemetry from becoming a stream of low-value events.

The objective is therefore not to **record everything**, but to record **the right things**.

A smaller number of well-structured records can be considerably more useful than a large volume of unstructured logs.

---

## Hot Path vs Cold Path

Telemetry explicitly separates recording from processing.

```text
                 HOT PATH
                     │
                     │ Scope / Metric
                     ▼
             ┌─────────────────┐
             │  ScopeRecord    │
             └────────┬────────┘
                      │
                      │ handoff
                      ▼
                ┌───────────┐
                │  Channel  │
                └─────┬─────┘
                      │
                      ▼
                 COLD PATH
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
      Resolve      Process      Persist
      Metadata     Records      / Output
```

The hot path is responsible only for capturing the information that must be retained.

The cold path can then perform work such as:

- resolving identifiers into metadata;
- formatting records;
- routing records;
- aggregating or enriching data;
- writing to files, stdout, databases, or other sinks.

This separation keeps instrumentation predictable while allowing the backend to remain extensible.

---

## Memory Layout

A central property of Telemetry is that the core record has a **fixed memory footprint**.

`ScopeRecord` stores its metrics inline using:

```rust
ArrayVec<Metric, 14>
```

No heap allocation is required to store metrics within a record.

The current layout is deliberately constrained:

- `Metric` — **16 bytes**
- `ScopeRecord` — **256 bytes**
- `ScopeRecord` therefore occupies exactly **4 × 64-byte cache lines**
- up to **14 metrics** can be stored inline

This makes the memory cost of instrumentation explicit and bounded.

The fixed-size record also makes it possible to reason about the cost of moving records between the hot and cold paths.

### Payloads

Both `ScopeRecord` and `Metric` expose an additional `u32` payload.

The payload is intentionally opaque to Telemetry: its interpretation is entirely application-defined.

This provides a compact escape hatch for additional information without introducing another dynamically sized field.

For example, a payload could represent:

- an application-specific identifier;
- a small enum;
- a sequence number;
- a compact status code;
- an index into application-owned data.

The payload also makes use of otherwise unavoidable alignment constraints, allowing additional information to be carried with no additional memory cost.

---

## Scope Lifecycle

`Scope` is an RAII instrumentation handle.

A Scope can be initialized as a root:

```rust
let mut scope = Scope::init(
    INITIALIZATION,
    channel.sender,
    None,
);
```

and nested using `child`:

```rust
let mut child_scope = scope.child(
    DATABASE,
    None,
);
```

Metrics can then be recorded against the current Scope:

```rust
child_scope.metric(
    END_TIME,
    0,
    None,
);
```

When a Scope is completed, its record is handed off to the cold path.

Because the record is **flushed before the Scope is dropped**, a long-lived Scope does not need to retain every record generated during its lifetime.

Conceptually:

```text
Scope
  │
  ├── record metrics
  │
  ├── flush record ──────► Channel
  │
  ├── reuse record
  │
  ├── record more metrics
  │
  └── flush record ──────► Channel
```

This allows multiple records to be emitted over the lifetime of a Scope without requiring an ever-growing collection of records.

---

## Usage

```rust
use telemetry::tracing::{
    capture::Scope,
    enrichment::Registry,
    handoff::{Channel, Subscriber},
    writers::{FileWriter, StdoutWriter},
};
use telemetry::schema::{metrics, scopes};

// ------ SETUP ------

// 1. Create the registry, channel, and subscriber.
let registry = Registry::default();

let channel = Channel::unbounded();

let mut subscriber = Subscriber::new(
    channel.receiver,
    &registry,
);

// 2. Register the writers that should receive resolved records.
subscriber.add_writer(StdoutWriter);

// ------ INSTRUMENTATION ------

// 1. Create a root scope.
let mut scope = Scope::init(
    scopes::INITIALIZATION,
    channel.sender,
    None,
);

foo();

// 2. Create a nested scope.
let mut child_scope = scope.child(
    scopes::INITIALIZATION,
    None,
);

bar();

// 3. Record metrics against the scope.
child_scope.metric(
    metrics::END_TIME,
    0,
    None,
);

// ------ SHUTDOWN / FLUSH ------

// Scopes can be explicitly dropped when their lifetime ends.
drop(scope);

// Drain the handoff channel and send records to the writers.
subscriber.drain();
```

The important part of the API is that the application-facing code remains small:

```text
Scope → Metric → flush
```

while the expensive work happens after the handoff:

```text
record → resolve → process → write
```

---

## Architecture

Telemetry is divided into a small number of responsibilities:

```text
┌─────────────────────────────────────────────────────┐
│                     Application                     │
│                                                     │
│   Scope ──────► Metric ──────► ScopeRecord          │
└─────────────────────────────┬───────────────────────┘
                              │
                              │ handoff
                              ▼
┌─────────────────────────────────────────────────────┐
│                    Cold Path                        │
│                                                     │
│  Channel → Subscriber → Resolution → Writers       │
│                                                     │
│                         ┌────► Stdout               │
│                         ├────► File                 │
│                         └────► Database             │
└─────────────────────────────────────────────────────┘
```

The architecture intentionally keeps the boundary between these two worlds explicit.

The hot path owns **capture**.

The cold path owns **interpretation and persistence**.

---

## When to use Telemetry

Telemetry is designed for systems where instrumentation overhead itself matters.

It is particularly appropriate when:

- latency matters more than convenience;
- allocation behavior must be predictable;
- records need a bounded memory footprint;
- telemetry must be captured from very hot code paths;
- processing can be deferred;
- structured execution context is more useful than free-form logs.

It is probably **not** the right choice if you primarily need:

- rich ecosystem integrations;
- dynamic log configuration;
- highly flexible structured logging;
- distributed tracing interoperability;
- automatic instrumentation of third-party libraries.

For those use cases, established frameworks such as [`tracing`](https://docs.rs/tracing) or OpenTelemetry are likely a better fit.

Telemetry deliberately chooses a narrower problem.

> **It is not trying to make observability easier. It is trying to make observability predictable.**
