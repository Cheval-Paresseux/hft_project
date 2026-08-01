use std::ops::{Add, AddAssign, Sub, SubAssign};

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

// ── Timestamp ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn new(val: u64) -> Self {
        Self(val)
    }
    pub fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for Timestamp {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<Timestamp> for u64 {
    fn from(val: Timestamp) -> u64 {
        val.0
    }
}

// ── Quantity ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quantity(u64);

impl Quantity {
    pub fn new(val: u64) -> Self {
        Self(val)
    }

    pub fn get(self) -> u64 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl Add for Quantity {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        self.0.checked_add(rhs.0).map(Self)
    }
}

impl Sub for Quantity {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl AddAssign for Quantity {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self
            .0
            .checked_add(rhs.0)
            .expect("Quantity overflow: invariant violated");
    }
}

impl SubAssign for Quantity {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self
            .0
            .checked_sub(rhs.0)
            .expect("Quantity underflow: invariant violated");
    }
}

impl From<u64> for Quantity {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<Quantity> for u64 {
    fn from(val: Quantity) -> u64 {
        val.0
    }
}

// ── Price ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Price(u64);

impl Price {
    pub fn new(val: u64) -> Self {
        Self(val)
    }
    pub fn get(self) -> u64 {
        self.0
    }
}

impl Add for Price {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        self.0.checked_add(rhs.0).map(Self)
    }
}

impl Sub for Price {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl AddAssign for Price {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self
            .0
            .checked_add(rhs.0)
            .expect("Price overflow: invariant violated");
    }
}

impl SubAssign for Price {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self
            .0
            .checked_sub(rhs.0)
            .expect("Price underflow: invariant violated");
    }
}

impl From<u64> for Price {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<Price> for u64 {
    fn from(val: Price) -> u64 {
        val.0
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
