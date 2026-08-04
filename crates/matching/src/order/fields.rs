use orderbook::order::Price;

// ── Order Type ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OrderKind {
    Market,
    Limit,
    Stop(Price),
    StopLimit(Price),
}

// ── Time In Force ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimeInForce {
    Day,
    GoodTillCanceled,
    FillOrKill,
    ImmediateOrCancel,
}
