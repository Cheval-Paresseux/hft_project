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

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let quantity = Quantity::new(10);

        assert_eq!(quantity.get(), 10);
    }

    #[test]
    fn get() {
        let quantity = Quantity::new(10);

        assert_eq!(quantity.get(), 10);
    }

    #[test]
    fn is_zero() {
        assert!(Quantity::new(0).is_zero());
        assert!(!Quantity::new(1).is_zero());
    }

    #[test]
    fn add() {
        assert_eq!(Quantity::new(10) + Quantity::new(5), Some(Quantity::new(15)));
        assert_eq!(Quantity::new(u64::MAX) + Quantity::new(1), None);
    }

    #[test]
    fn sub() {
        assert_eq!(Quantity::new(10) - Quantity::new(5), Some(Quantity::new(5)));
        assert_eq!(Quantity::new(5) - Quantity::new(10), None);
    }
}
