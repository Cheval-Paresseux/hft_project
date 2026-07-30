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
