use book::level3::L3BookUpdate;

use smallvec::SmallVec;

// ── Instructions ──────────────────────────────────────────────────────────────

const BOOK_UPDATES_CAP: usize = 16;
const TRIGGER_UPDATES_CAP: usize = 4;
const OUTPUTS_CAP: usize = 8;
const FOLLOW_UPS_CAP: usize = 2;

pub type BookUpdates<I> = SmallVec<[L3BookUpdate<I>; BOOK_UPDATES_CAP]>;
pub type TriggerUpdates<U> = SmallVec<[U; TRIGGER_UPDATES_CAP]>;
pub type Outputs<O> = SmallVec<[O; OUTPUTS_CAP]>;
pub type FollowUps<F> = SmallVec<[F; FOLLOW_UPS_CAP]>;

/// What the protocol decided for one message:
///
/// - `book_updates`: applied to the order book,
/// - `trigger_updates`: applied to the trigger books,
/// - `outputs`: pushed to the output queue once the updates are applied,
/// - `follow_ups`: messages processed next, against the updated books
///   (triggered stop orders, auction uncrossing, …).
///
/// Stored inline up to small capacities, so a regular message does not
/// allocate.
#[derive(Debug, PartialEq, Eq)]
pub struct Instructions<I, U, O, F> {
    pub book_updates: BookUpdates<I>,
    pub trigger_updates: TriggerUpdates<U>,
    pub outputs: Outputs<O>,
    pub follow_ups: FollowUps<F>,
}

impl<I, U, O, F> Instructions<I, U, O, F> {
    pub fn new() -> Self {
        Self {
            book_updates: SmallVec::new(),
            trigger_updates: SmallVec::new(),
            outputs: SmallVec::new(),
            follow_ups: SmallVec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.book_updates.is_empty()
            && self.trigger_updates.is_empty()
            && self.outputs.is_empty()
            && self.follow_ups.is_empty()
    }
}

impl<I, U, O, F> Default for Instructions<I, U, O, F> {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    type TestInstructions = Instructions<u64, u64, u64, u64>;

    #[test]
    fn new() {
        let instructions = TestInstructions::new();

        assert!(instructions.book_updates.is_empty());
        assert!(instructions.trigger_updates.is_empty());
        assert!(instructions.outputs.is_empty());
        assert!(instructions.follow_ups.is_empty());
    }

    #[test]
    fn is_empty() {
        let mut instructions = TestInstructions::new();

        assert!(instructions.is_empty());

        instructions.follow_ups.push(1);

        assert!(!instructions.is_empty());
    }
}
