use std::ops::{Add, Sub};

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

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

// ── Add & Sub Trait ───────────────────────────────────────────────────────────

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

// ── From Trait ────────────────────────────────────────────────────────────────

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
