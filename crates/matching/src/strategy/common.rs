// ── Execution Mode ────────────────────────────────────────────────────────────

pub trait ExecutionMode {}

// ── Allocation Algorithm ──────────────────────────────────────────────────────

pub trait AllocationAlgorithm {}

// ── Matching Strategy ─────────────────────────────────────────────────────────

pub trait MatchingStrategy {
    type Execution: ExecutionMode;
    type Allocation: AllocationAlgorithm;
}
