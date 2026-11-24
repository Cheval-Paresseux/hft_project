// ── Matching Queue ────────────────────────────────────────────────────────────

pub trait MatchingQueue {
    type Message;
}

// ── Input Queue ───────────────────────────────────────────────────────────────

pub trait InputQueue: MatchingQueue {
    fn pop(&mut self) -> Option<Self::Message>;
}

// ── Output Queue ──────────────────────────────────────────────────────────────

pub trait OutputQueue: MatchingQueue {
    fn push(&mut self, message: Self::Message);
}
