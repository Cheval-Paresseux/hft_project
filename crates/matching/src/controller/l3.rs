use super::common::{BookController, BookInstruction, BookRouting, InstructionsBatch};

use crate::common::MatchingError;

use book::level3::L3OrderBook;

// ── L3 Book Controller ────────────────────────────────────────────────────────

pub struct L3Controller<B> {
    book: B,
}

impl<B> L3Controller<B> {
    pub fn new(book: B) -> Self {
        Self { book }
    }
}

impl<B> BookController for L3Controller<B>
where
    B: L3OrderBook,
{
    type Order = B::Incoming;
    type Book = B;

    fn view(&self) -> &Self::Book {
        &self.book
    }

    fn execute(&mut self, batch: InstructionsBatch<Self::Order>) -> Result<(), MatchingError> {
        debug_assert_eq!(batch.routing, BookRouting::default());

        for instruction in batch {
            match instruction {
                BookInstruction::Insert(order) => self.book.insert(order)?,
                BookInstruction::Remove(order_id) => self.book.remove(order_id)?,
                BookInstruction::Replace(old_order_id, new_order) => self.book.replace(old_order_id, new_order)?,
                BookInstruction::Reduce(order_id, remaining_quantity) => {
                    self.book.reduce(order_id, remaining_quantity)?
                }
                BookInstruction::Increase(order_id, remaining_quantity) => {
                    self.book.increase(order_id, remaining_quantity)?
                }
            }
        }

        Ok(())
    }
}
