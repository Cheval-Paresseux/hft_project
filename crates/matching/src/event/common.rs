use types::order::{
    fields::{OrderId, Price, Quantity},
    orders::LimitOrder,
};

// ── Market Event ──────────────────────────────────────────────────────────────

/// Incoming engine input — what happened "outside" and must be planned.
///
/// Provisional variant set: other order kinds (market, stop, stop-limit) and
/// trigger events are added when those paths land.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarketEvent {
    New(LimitOrder),
    Cancel(OrderId),
    Replace(OrderId, LimitOrder),
    Uncross,
}

// ── Fill ──────────────────────────────────────────────────────────────────────

/// What an `AllocationAlgorithm` produces: an order partially (or fully)
/// consumed by a given quantity. The execution mode turns fills into both
/// instructions (`Reduce`/`Remove`) and events (`Executed`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fill {
    pub order_id: OrderId,
    pub quantity: Quantity,
}

// ── Matching Event ────────────────────────────────────────────────────────────

/// Trading semantics emitted alongside a plan — what the OMS, market data and
/// telemetry consume. Never used to mutate a book.
///
/// Provisional variant set — grows with the algorithms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchingEvent {
    Executed {
        maker: OrderId,
        taker: OrderId,
        price: Price,
        quantity: Quantity,
    },
    Resting {
        order_id: OrderId,
    },
}
