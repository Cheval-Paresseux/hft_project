use super::instructions::Instructions;
use crate::trigger::TriggerBooks;

use book::{BookError, level3::L3BookView};

use types::order::{OrderError, orders::BookableOrder};

use thiserror::Error;

// ── Plan Error ────────────────────────────────────────────────────────────────

/// A message could not be planned by the protocol.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum PlanError {
    #[error(transparent)]
    OrderError(#[from] OrderError),

    #[error(transparent)]
    BookError(#[from] BookError),
}

// ── Matching Protocol ─────────────────────────────────────────────────────────

/// The matching algorithm: continuous trading, call auction, …
///
/// Takes an input message and read-only views of the order book and the
/// trigger books, and returns the instructions to apply. It never mutates the
/// books — the matching engine applies the instructions once planning is over.
///
/// The protocol states what it needs from the books through bounds on `B` and
/// `T`, so the same protocol runs on any book and any trigger set that fits.
///
/// Refusing a message is a normal outcome, written as an output; an error
/// means the message could not be planned at all.
pub trait MatchingProtocol<B: L3BookView, T: TriggerBooks> {
    /// The messages this protocol consumes, including its own follow-ups.
    type Input;

    /// The messages this protocol produces.
    type Output;

    /// The orders this protocol rests in the order book.
    type Incoming: BookableOrder;

    fn plan(
        &mut self,
        message: &Self::Input,
        book: &B,
        triggers: &T,
    ) -> Result<ProtocolInstructions<Self, B, T>, PlanError>;
}

/// The instructions returned by a protocol `P` over a book `B` and a trigger
/// set `T`.
pub type ProtocolInstructions<P, B, T> = Instructions<
    <P as MatchingProtocol<B, T>>::Incoming,
    <T as TriggerBooks>::Update,
    <P as MatchingProtocol<B, T>>::Output,
    <P as MatchingProtocol<B, T>>::Input,
>;
