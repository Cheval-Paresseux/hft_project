use crate::common::MatchingError;

use types::order::{
    fields::{OrderId, Quantity},
    orders::IncomingOrder,
};

use smallvec::SmallVec;

// ── Instructions ──────────────────────────────────────────────────────────────

pub struct BookRouting {}

pub enum BookInstruction<I: IncomingOrder> {
    Insert(I),
    Remove(OrderId),
    Replace(OrderId, I),
    Reduce(OrderId, Quantity),
    Increase(OrderId, Quantity),
}

pub struct InstructionsBatch<I: IncomingOrder> {
    pub routing: BookRouting,
    instructions: SmallVec<[BookInstruction<I>; 8]>,
}

impl<I: IncomingOrder> InstructionsBatch<I> {
    pub fn new(routing: BookRouting) -> Self {
        Self {
            instructions: SmallVec::new(),
            routing,
        }
    }

    pub fn push(&mut self, instruction: BookInstruction<I>) {
        self.instructions.push(instruction);
    }

    pub fn iter(&self) -> impl Iterator<Item = &BookInstruction<I>> {
        self.instructions.iter()
    }
}

// ── Controller ────────────────────────────────────────────────────────────────

pub trait BookController {
    type Order: IncomingOrder;

    fn execute(&mut self, batch: InstructionsBatch<Self::Order>) -> Result<(), MatchingError>;
}
