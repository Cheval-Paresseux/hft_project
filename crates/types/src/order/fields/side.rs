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

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_u4() {
        assert_eq!(OrderSide::Bid.as_u4(), 0);
        assert_eq!(OrderSide::Ask.as_u4(), 1);
    }

    #[test]
    fn opposite() {
        assert_eq!(OrderSide::Bid.opposite(), OrderSide::Ask);
        assert_eq!(OrderSide::Ask.opposite(), OrderSide::Bid);
    }
}
