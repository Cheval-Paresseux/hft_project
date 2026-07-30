use super::trade::{ Timestamp, Price, OrderSide };
use std::fmt;
use std::ops;
use arrayvec::ArrayString;

const CLIENT_CAP: usize = 32;

// ── Client Order ID ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClientOrderId(ArrayString<CLIENT_CAP>);

impl ClientOrderId {
    pub fn new(id: &str) -> Option<Self> {
        ArrayString::try_from(id).ok().map(Self)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ops::Deref for ClientOrderId {
    type Target = str;
    fn deref(&self) -> &str { self.0.as_str() }
}

impl fmt::Display for ClientOrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── Order ID ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OrderId(u64);

impl OrderId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn from_fields(timestamp: Timestamp, price: Price, side: OrderSide) -> Option<Self> {
        if timestamp.get() > 0xFFFF_FFFF_FFFF { return None; }
        Some(Self(
            (timestamp.get() << 16) | ((price.get() & 0xFFF) << 4) | (side.as_u4() & 0xF)
        ))
    }

    pub fn timestamp(&self) -> u64 { self.0 >> 16 }
    pub fn price(&self)     -> u64 { (self.0 >> 4) & 0xFFF }
    pub fn side(&self)      -> u64 { self.0 & 0xF }
}

impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OrderId(ts={}, price={}, side={})",
            self.timestamp(), self.price(), self.side())
    }
}

// ── Modification ID ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModificationId(u64);

impl ModificationId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for ModificationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ModificationId ={}", self.0)
    }
}