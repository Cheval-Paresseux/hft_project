# Matching

Matcher interfaces: queues, protocol, books, and the workflow tying them together.

## Overview

Orders arrive in parallel on several ports, are linearized by a sequencer, and dispatched to N matchers. The `matching` crate defines what a matcher is, without committing to any implementation: declaring a matcher means choosing each of its components.

| Component | Trait | Role |
|---|---|---|
| Input queue | `InputQueue` | Sequenced messages to process |
| Output queue | `OutputQueue` | Messages produced by the matcher |
| Protocol | `MatchingProtocol` | The matching algorithm: message + book views → instructions |
| Order book | `book::level3::L3OrderBook` | Resting orders |
| Trigger books | `TriggerBooks` | Any set of trigger books (stops, timestamps, …), possibly empty |

## Architecture

### Workflow

Every message follows the same cycle:

```text
input ──► plan ───────────► execute ─────────► output
  ▲       MatchingProtocol   MatchingEngine
  │       (read-only books)  (mutable books)
  └──────── follow-ups ◄────────┘
```

- **`MatchingEngine`** declares the components as associated types and exposes them through `components()`. The workflow — `poll`, `process`, `step`, `plan`, `execute` — is provided and can be overridden.
- **`Instructions`** hold what the protocol decided: order book updates, trigger book updates, outputs, and follow-up messages processed next against the updated books (triggered stop orders, auction uncrossing, …). They are stored inline up to small capacities, so a regular message does not allocate.

Planning only reads the books, so a planning failure on the message itself leaves them untouched. Any other failure leaves them partially updated, and the matcher must stop.

### Protocol

`MatchingProtocol<B, T>` is generic over the order book view `B` and the trigger set `T`: a protocol states what it needs through bounds on them, and runs on any books that fit. It fixes the message types (`Input`, `Output`) and the orders it rests (`Incoming`); the engine requires its queues and book to agree.

Refusing a message is a normal outcome, written as an output; `PlanError` means a message could not be planned at all.

### Trigger Books

`TriggerBooks` is implemented for `()` and tuples of up to four trigger books. A tuple is addressed positionally through `TriggerUpdate` (`T0` → `.0`, `T1` → `.1`, …). A named struct with its own update enum works just as well.

## Crate Structure

```text
matching
├── engine
│   └── common         # MatchingEngine, Components, MatchingError
│
├── protocol
│   ├── common         # MatchingProtocol, PlanError
│   └── instructions   # Instructions
│
├── queue
│   └── common         # MatchingQueue, InputQueue, OutputQueue
│
└── trigger
    ├── common         # TriggerBooks, TriggerUpdate
    └── tuple          # TriggerBooks for () and tuples
```

## Development Roadmap

- [x] **Interfaces:** Engine, protocol, queues, trigger books.
- [ ] **Queues:** Bounded SPSC ring buffers.
- [ ] **Continuous trading protocol:** Limit, market, and stop orders.
- [ ] **Allocation rules:** Price-time, pro-rata.
- [ ] **Call auction protocol.**
