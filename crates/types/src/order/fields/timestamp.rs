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

// ── From Trait ────────────────────────────────────────────────────────────────

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

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let timestamp = Timestamp::new(100_000);

        assert_eq!(timestamp.get(), 100_000);
    }

    #[test]
    fn get() {
        let timestamp = Timestamp::new(u64::MAX);

        assert_eq!(timestamp.get(), u64::MAX);
    }
}
