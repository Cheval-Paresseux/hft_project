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
    #[test]
    fn process_cpu_time_snapshot() {
        let before = super::process_cpu_time_snapshot();

        let mut acc = 0u64;
        for i in 0..100_000 {
            acc = acc.wrapping_add(i);
        }
        std::hint::black_box(acc);

        let after = super::process_cpu_time_snapshot();

        assert!(after.user_time >= before.user_time);
        assert!(after.system_time >= before.system_time);
    }

    #[test]
    fn process_user_cpu_time() {
        let before = super::process_cpu_time_snapshot();

        let value = super::process_user_cpu_time();

        let after = super::process_cpu_time_snapshot();

        assert!(value >= before.user_time);
        assert!(value <= after.user_time);
    }

    #[test]
    fn process_system_cpu_time() {
        let before = super::process_cpu_time_snapshot();

        let value = super::process_system_cpu_time();

        let after = super::process_cpu_time_snapshot();

        assert!(value >= before.system_time);
        assert!(value <= after.system_time);
    }
}
