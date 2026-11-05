use procfs::process::Process;

// ── Memory usage ──────────────────────────────────────────────────────────────

#[inline(always)]
pub fn process_memory_snapshot() -> ProcessMemory {
    let status = status();

    ProcessMemory {
        resident: status.vmrss.unwrap_or(0) * 1024,
        resident_peak: status.vmhwm.unwrap_or(0) * 1024,
        virtual_: status.vmsize.unwrap_or(0) * 1024,
        virtual_peak: status.vmpeak.unwrap_or(0) * 1024,
        swap: status.vmswap.unwrap_or(0) * 1024,
        locked: status.vmlck.unwrap_or(0) * 1024,
    }
}

#[inline(always)]
fn status() -> procfs::process::Status {
    Process::myself()
        .expect("failed to get current process")
        .status()
        .expect("failed to read process status")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessMemory {
    pub resident: u64,
    pub resident_peak: u64,
    pub virtual_: u64,
    pub virtual_peak: u64,
    pub swap: u64,
    pub locked: u64,
}

// ── Individual getters ────────────────────────────────────────────────────────

#[inline(always)]
pub fn process_resident_memory() -> u64 {
    process_memory_snapshot().resident
}

#[inline(always)]
pub fn process_resident_memory_peak() -> u64 {
    process_memory_snapshot().resident_peak
}

#[inline(always)]
pub fn process_virtual_memory() -> u64 {
    process_memory_snapshot().virtual_
}

#[inline(always)]
pub fn process_virtual_memory_peak() -> u64 {
    process_memory_snapshot().virtual_peak
}

#[inline(always)]
pub fn process_swap_memory() -> u64 {
    process_memory_snapshot().swap
}

#[inline(always)]
pub fn process_locked_memory() -> u64 {
    process_memory_snapshot().locked
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    fn assert_page_aligned(value: u64) {
        assert_eq!(value % 1024, 0, "value {value} is not page aligned");
    }

    #[test]
    fn process_memory_snapshot() {
        let snapshot = super::process_memory_snapshot();

        assert!(snapshot.resident <= snapshot.resident_peak);
        assert!(snapshot.virtual_ <= snapshot.virtual_peak);

        assert_page_aligned(snapshot.resident);
        assert_page_aligned(snapshot.resident_peak);
        assert_page_aligned(snapshot.virtual_);
        assert_page_aligned(snapshot.virtual_peak);
        assert_page_aligned(snapshot.swap);
        assert_page_aligned(snapshot.locked);

        assert!(snapshot.resident > 0);
        assert!(snapshot.virtual_ > 0);
        assert!(snapshot.virtual_peak > 0);
    }

    #[test]
    fn process_resident_memory() {
        let value = super::process_resident_memory();

        assert_page_aligned(value);
        assert!(value > 0);

        let after = super::process_memory_snapshot();
        assert!(value <= after.resident_peak);
    }

    #[test]
    fn process_resident_memory_peak() {
        let before = super::process_memory_snapshot();

        let value = super::process_resident_memory_peak();

        assert_page_aligned(value);
        assert!(value >= before.resident_peak);
        assert!(value >= before.resident);
    }

    #[test]
    fn process_virtual_memory() {
        let value = super::process_virtual_memory();

        assert_page_aligned(value);
        assert!(value > 0);

        let after = super::process_memory_snapshot();
        assert!(value <= after.virtual_peak);
    }

    #[test]
    fn process_virtual_memory_peak() {
        let before = super::process_memory_snapshot();

        let value = super::process_virtual_memory_peak();

        assert_page_aligned(value);
        assert!(value >= before.virtual_peak);
        assert!(value >= before.virtual_);
    }

    #[test]
    fn process_swap_memory() {
        let value = super::process_swap_memory();

        assert_page_aligned(value);
    }

    #[test]
    fn process_locked_memory() {
        let value = super::process_locked_memory();

        assert_page_aligned(value);
    }
}
