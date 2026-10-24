// ── OrderSide ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OrderSide {
    Bid,
    Ask,
}

impl OrderSide {
    pub fn as_u4(self) -> u64 {
        match self {
            OrderSide::Bid => 0,
            OrderSide::Ask => 1,
        }
    }

    pub fn opposite(self) -> OrderSide {
        match self {
            OrderSide::Bid => OrderSide::Ask,
            OrderSide::Ask => OrderSide::Bid,
        }
    }
}
