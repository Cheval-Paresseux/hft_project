use crate::controller::{BookInstruction, BookRouting, InstructionsBatch};
use crate::event::MatchingEvent;

use types::order::orders::IncomingOrder;

use smallvec::SmallVec;

// ── Plan ──────────────────────────────────────────────────────────────────────

/// The artifact of one matching decision: a batch of book instructions (the
/// write path) and the trading events derived from the same decision (the
/// messaging path).
///
/// Fields are private so the storage can be reworked (single buffer, arena
/// indices, …) without touching any strategy implementation.
#[derive(Debug, PartialEq, Eq)]
pub struct Plan<I: IncomingOrder> {
    batch: InstructionsBatch<I>,
    events: SmallVec<[MatchingEvent; 4]>,
}

impl<I: IncomingOrder> Plan<I> {
    pub fn new() -> Self {
        Self {
            batch: InstructionsBatch::new(BookRouting::default()),
            events: SmallVec::new(),
        }
    }

    pub fn push_instruction(&mut self, instruction: BookInstruction<I>) {
        self.batch.push(instruction);
    }

    pub fn push_event(&mut self, event: MatchingEvent) {
        self.events.push(event);
    }

    pub fn instructions(&self) -> &InstructionsBatch<I> {
        &self.batch
    }

    pub fn events(&self) -> &[MatchingEvent] {
        &self.events
    }

    pub fn into_parts(self) -> (InstructionsBatch<I>, SmallVec<[MatchingEvent; 4]>) {
        (self.batch, self.events)
    }
}

impl<I: IncomingOrder> Default for Plan<I> {
    fn default() -> Self {
        Self::new()
    }
}
