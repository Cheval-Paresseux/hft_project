use crate::controller::common::BookController;
use crate::strategy::common::MatchingStrategy;

// ── Matching Engine ───────────────────────────────────────────────────────────

pub struct MatchingEngine<C: BookController, S: MatchingStrategy> {
    controller: C,
    strategy: S,
}
