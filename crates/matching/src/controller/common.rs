use crate::common::MatchingError;

use book::level3::L3OrderBook;

use types::order::{
    fields::{OrderId, Quantity},
    orders::IncomingOrder,
};

use smallvec::SmallVec;

// ── Book Routing ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BookRouting {}

// ── Instructions ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BookInstruction<I: IncomingOrder> {
    Insert(I),
    Remove(OrderId),
    Replace(OrderId, I),
    Reduce(OrderId, Quantity),
    Increase(OrderId, Quantity),
}

// ── Instructions Batch ────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
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

impl<I: IncomingOrder> IntoIterator for InstructionsBatch<I> {
    type Item = BookInstruction<I>;
    type IntoIter = smallvec::IntoIter<[BookInstruction<I>; 8]>;

    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}

// ── Book Controller ───────────────────────────────────────────────────────────

pub trait BookController {
    type Order: IncomingOrder;
    type Book: L3OrderBook<Incoming = Self::Order>;

    fn view(&self) -> &Self::Book;
    fn execute(&mut self, batch: InstructionsBatch<Self::Order>) -> Result<(), MatchingError>;
}
