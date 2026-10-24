use std::ops::{Add, Sub};

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

// ── Add & Sub Trait ───────────────────────────────────────────────────────────

impl Add for Price {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }
}

impl Sub for Price {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

// ── From Trait ────────────────────────────────────────────────────────────────

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
