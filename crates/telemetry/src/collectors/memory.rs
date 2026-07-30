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
    use super::*;

    #[test]
    fn resident_memory_is_not_greater_than_peak() {
        let snapshot = process_memory_snapshot();

        assert!(snapshot.resident <= snapshot.resident_peak);
    }

    #[test]
    fn virtual_memory_is_not_greater_than_peak() {
        let snapshot = process_memory_snapshot();

        assert!(snapshot.virtual_ <= snapshot.virtual_peak);
    }

    #[test]
    fn all_values_are_page_aligned() {
        let snapshot = process_memory_snapshot();

        assert_eq!(snapshot.resident % 1024, 0);
        assert_eq!(snapshot.resident_peak % 1024, 0);
        assert_eq!(snapshot.virtual_ % 1024, 0);
        assert_eq!(snapshot.virtual_peak % 1024, 0);
        assert_eq!(snapshot.swap % 1024, 0);
        assert_eq!(snapshot.locked % 1024, 0);
    }

    #[test]
    fn snapshot_is_self_consistent() {
        let first = process_memory_snapshot();
        let second = process_memory_snapshot();

        assert!(second.resident_peak >= first.resident_peak);
        assert!(second.virtual_peak >= first.virtual_peak);
    }

    #[test]
    fn process_has_some_virtual_memory() {
        let snapshot = process_memory_snapshot();

        assert!(snapshot.virtual_ > 0);
        assert!(snapshot.virtual_peak > 0);
    }
}
