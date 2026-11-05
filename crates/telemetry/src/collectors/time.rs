use std::{
    sync::LazyLock,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

// ── Monotonic Clock ───────────────────────────────────────────────────────────

pub static CLOCK: LazyLock<Clock> = LazyLock::new(Clock::default);

pub struct Clock {
    unix_nanos: u64,
    instant: Instant,
}

impl Clock {
    #[inline(always)]
    pub fn unix_nanos(&self) -> u64 {
        self.unix_nanos + self.instant.elapsed().as_nanos() as u64
    }
}

impl Default for Clock {
    fn default() -> Self {
        let instant = Instant::now();
        let system = SystemTime::now();

        let unix_nanos = system.duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

        Self {
            unix_nanos,
            instant,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[inline(always)]
pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[inline(always)]
pub fn instant() -> Instant {
    Instant::now()
}

#[inline(always)]
pub fn duration(first: Instant, last: Instant) -> u64 {
    (last - first).as_nanos() as u64
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::Clock;
    use std::time::{SystemTime, UNIX_EPOCH};

    const CLOCK_TOLERANCE_NANOS: u64 = 1_000_000_000;

    fn system_nanos() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    #[test]
    fn default() {
        let clock = Clock::default();

        assert!(clock.unix_nanos() > 0);
        assert!(clock.unix_nanos().abs_diff(system_nanos()) < CLOCK_TOLERANCE_NANOS);
    }

    #[test]
    fn unix_nanos() {
        let clock = Clock::default();

        let first = clock.unix_nanos();
        let second = clock.unix_nanos();

        assert!(second >= first);
        assert!(first.abs_diff(system_nanos()) < CLOCK_TOLERANCE_NANOS);
    }

    #[test]
    fn timestamp() {
        let before = system_nanos();

        let value = super::timestamp();

        let after = system_nanos();

        assert!(value >= before);
        assert!(value <= after);
    }

    #[test]
    fn instant() {
        let first = super::instant();
        let second = super::instant();

        assert!(second >= first);
    }

    #[test]
    fn duration() {
        let first = super::instant();

        let mut acc = 0u64;
        for i in 0..100_000 {
            acc = acc.wrapping_add(i);
        }
        std::hint::black_box(acc);

        let last = super::instant();

        assert!(super::duration(first, last) > 0);
        assert_eq!(super::duration(first, first), 0);
    }
}
