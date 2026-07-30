
// ── Book Config ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct BookConfig {
    pub lookup_capacity: usize,
    pub bid_side_capacity: usize,
    pub ask_side_capacity: usize,
}

impl BookConfig {
    pub fn new(lookup_capacity: usize, bid_side_capacity: usize, ask_side_capacity: usize) -> Self {
        Self { lookup_capacity, bid_side_capacity, ask_side_capacity }
    }
}

// ── Side Config ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct SideConfig {
    pub level_capacity: usize,
}

impl SideConfig {
    pub fn new(level_capacity: usize) -> Self {
        Self { level_capacity }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════
