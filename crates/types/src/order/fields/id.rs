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
