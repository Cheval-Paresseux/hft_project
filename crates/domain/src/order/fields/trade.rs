use std::fmt;

// ── Duration ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(u64);

impl Duration {
    pub fn new(val: u64) -> Self { Self(val) }
    pub fn get(self) -> u64 { self.0 }
    pub fn is_zero(self) -> bool { self.0 == 0 }
}

impl From<u64> for Duration {
    fn from(val: u64) -> Self { Self(val) }
}

impl From<Duration> for u64 {
    fn from(val: Duration) -> u64 { val.0 }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── Timestamp ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn new(val: u64) -> Self { Self(val) }
    pub fn get(self) -> u64 { self.0 }

    pub fn advance(self, duration: Duration) -> Option<Timestamp> {
        self.0.checked_add(duration.0).map(Timestamp)
    }

    pub fn rewind(self, duration: Duration) -> Option<Timestamp> {
        self.0.checked_sub(duration.0).map(Timestamp)
    }

    pub fn delta(self, other: Timestamp) -> Duration {
        Duration(if self.0 >= other.0 { self.0 - other.0 } else { other.0 - self.0 })
    }
}

impl From<u64> for Timestamp {
    fn from(val: u64) -> Self { Self(val) }
}

impl From<Timestamp> for u64 {
    fn from(val: Timestamp) -> u64 { val.0 }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── Price ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Price(u64);

impl Price {
    pub fn new(val: u64) -> Self { Self(val) }
    pub fn get(self) -> u64 { self.0 }

    pub fn raise(self, amount: Price) -> Option<Price> {
        self.0.checked_add(amount.0).map(Price)
    }

    pub fn lower(self, amount: Price) -> Option<Price> {
        self.0.checked_sub(amount.0).map(Price)
    }

    pub fn delta(self, other: Price) -> Price {
        Price(if self.0 >= other.0 { self.0 - other.0 } else { other.0 - self.0 })
    }
}

impl From<u64> for Price {
    fn from(val: u64) -> Self { Self(val) }
}

impl From<Price> for u64 {
    fn from(val: Price) -> u64 { val.0 }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── Quantity ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quantity(u64);

impl Quantity {
    pub fn new(val: u64) -> Self { Self(val) }
    pub fn get(self) -> u64 { self.0 }

    pub fn add(self, amount: Quantity) -> Option<Quantity> {
        self.0.checked_add(amount.0).map(Quantity)
    }

    pub fn sub(self, amount: Quantity) -> Option<Quantity> {
        self.0.checked_sub(amount.0).map(Quantity)
    }

    pub fn delta(self, other: Quantity) -> Quantity {
        Quantity(if self.0 >= other.0 { self.0 - other.0 } else { other.0 - self.0 })
    }

    pub fn is_zero(self) -> bool { self.0 == 0 }
}

impl From<u64> for Quantity {
    fn from(val: u64) -> Self { Self(val) }
}

impl From<Quantity> for u64 {
    fn from(val: Quantity) -> u64 { val.0 }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderSide::Bid => write!(f, "Bid"),
            OrderSide::Ask => write!(f, "Ask"),
        }
    }
}

// ── Time In Force ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MarketTimeInForce {
    FillOrKill,
    ImmediateOrCancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LimitTimeInForce {
    Day,
    GoodTillCancel,
    FillOrKill,
    ImmediateOrCancel,
}

// ── Order Instruction ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OrderInstruction {
    Market { tif: MarketTimeInForce },
    Limit { limit: Price, tif: LimitTimeInForce },
    Stop { stop: Price, tif: LimitTimeInForce },
    StopLimit { stop: Price, limit: Price, tif: LimitTimeInForce },
}

