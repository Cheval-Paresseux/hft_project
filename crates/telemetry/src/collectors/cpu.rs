use procfs::process::Process;

// ── CPU usage ─────────────────────────────────────────────────────────────────

#[inline(always)]
pub fn process_cpu_time_snapshot() -> ProcessCpuTime {
    let stat = process_stat().expect("failed to read process cpu statistics");

    ProcessCpuTime {
        user_time: stat.utime,
        system_time: stat.stime,
    }
}

#[inline(always)]
fn process_stat() -> procfs::ProcResult<procfs::process::Stat> {
    Process::myself()?.stat()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessCpuTime {
    pub user_time: u64,
    pub system_time: u64,
}

// ── Individual getters ────────────────────────────────────────────────────────

#[inline(always)]
pub fn process_user_cpu_time() -> u64 {
    process_cpu_time_snapshot().user_time
}

#[inline(always)]
pub fn process_system_cpu_time() -> u64 {
    process_cpu_time_snapshot().system_time
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_times_are_monotonic() {
        let before = process_cpu_time_snapshot();

        let mut acc = 0u64;
        for i in 0..5_000_000 {
            acc = acc.wrapping_add(i);
        }
        std::hint::black_box(acc);

        let after = process_cpu_time_snapshot();

        assert!(after.user_time >= before.user_time);
        assert!(after.system_time >= before.system_time);
    }

    #[test]
    fn snapshot_is_consistent() {
        let first = process_cpu_time_snapshot();
        let second = process_cpu_time_snapshot();

        assert!(second.user_time >= first.user_time);
        assert!(second.system_time >= first.system_time);
    }
}
