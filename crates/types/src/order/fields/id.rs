use super::{price::Price, side::OrderSide, timestamp::Timestamp};

// ── Order ID ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OrderId(u64);

impl OrderId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn from_fields(timestamp: Timestamp, price: Price, side: OrderSide) -> Option<Self> {
        if timestamp.get() > 0xFFFF_FFFF_FFFF {
            return None;
        }

        Some(Self(
            (timestamp.get() << 16) | ((price.get() & 0xFFF) << 4) | (side.as_u4() & 0xF),
        ))
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

impl OrderId {
    pub fn timestamp(&self) -> u64 {
        self.0 >> 16
    }

    pub fn price(&self) -> u64 {
        (self.0 >> 4) & 0xFFF
    }

    pub fn side(&self) -> u64 {
        self.0 & 0xF
    }
}

// ── From Trait ────────────────────────────────────────────────────────────────

impl From<u64> for OrderId {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<OrderId> for u64 {
    fn from(val: OrderId) -> u64 {
        val.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn packed(side: OrderSide) -> OrderId {
        OrderId::from_fields(Timestamp::new(0x1234_5678_9ABC), Price::new(0xABC), side).unwrap()
    }

    #[test]
    fn new() {
        let id = OrderId::new(72);

        assert_eq!(id.get(), 72);
    }

    #[test]
    fn from_fields() {
        let id = packed(OrderSide::Ask);

        assert_eq!(id.timestamp(), 0x1234_5678_9ABC);
        assert_eq!(id.price(), 0xABC);
        assert_eq!(id.side(), 1);

        assert_eq!(
            OrderId::from_fields(Timestamp::new(0x1_0000_0000_0000), Price::new(0xABC), OrderSide::Ask),
            None
        );
    }

    #[test]
    fn get() {
        let id = OrderId::new(72);

        assert_eq!(id.get(), 72);
    }

    #[test]
    fn timestamp() {
        assert_eq!(packed(OrderSide::Bid).timestamp(), 0x1234_5678_9ABC);
    }

    #[test]
    fn price() {
        assert_eq!(packed(OrderSide::Bid).price(), 0xABC);
    }

    #[test]
    fn side() {
        assert_eq!(packed(OrderSide::Bid).side(), 0);
        assert_eq!(packed(OrderSide::Ask).side(), 1);
    }
}
