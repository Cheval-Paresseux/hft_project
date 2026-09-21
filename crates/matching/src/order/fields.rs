// ── Time In Force ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimeInForce {
    Day,
    GoodTillCanceled,
    FillOrKill,
    ImmediateOrCancel,
}

impl From<MarketTimeInForce> for TimeInForce {
    fn from(value: MarketTimeInForce) -> Self {
        match value {
            MarketTimeInForce::FillOrKill => TimeInForce::FillOrKill,
            MarketTimeInForce::ImmediateOrCancel => TimeInForce::ImmediateOrCancel,
        }
    }
}

// ── Market Time In Force ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MarketTimeInForce {
    FillOrKill,
    ImmediateOrCancel,
}

impl TryFrom<TimeInForce> for MarketTimeInForce {
    type Error = ();

    fn try_from(value: TimeInForce) -> Result<Self, Self::Error> {
        match value {
            TimeInForce::FillOrKill => Ok(Self::FillOrKill),
            TimeInForce::ImmediateOrCancel => Ok(Self::ImmediateOrCancel),
            TimeInForce::Day | TimeInForce::GoodTillCanceled => Err(()),
        }
    }
}
