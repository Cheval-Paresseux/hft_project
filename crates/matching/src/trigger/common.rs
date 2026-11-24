use book::{
    BookError,
    trigger::{TriggerBookUpdate, TriggerBookView},
};

use std::convert::Infallible;

// ── Trigger Books ─────────────────────────────────────────────────────────────

/// The set of trigger books owned by a matcher (stops, timestamps, …).
///
/// Protocols read the set through its concrete type (e.g. `triggers.0` for a
/// tuple) and address each book through the set's `Update` type. Implemented
/// for `()` and tuples of up to four trigger books; a named struct with its
/// own update enum works just as well.
pub trait TriggerBooks {
    type Update;

    fn apply(&mut self, update: Self::Update) -> Result<(), BookError>;
}

/// The update type of a single trigger book.
pub type TriggerBookUpdateOf<B> = TriggerBookUpdate<<B as TriggerBookView>::Threshold, <B as TriggerBookView>::Order>;

// ── Trigger Update ────────────────────────────────────────────────────────────

/// Positional update for a tuple of trigger books: `T0` targets `.0`, `T1`
/// targets `.1`, … Unused positions default to `Infallible`, so they can never
/// be built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerUpdate<U0, U1 = Infallible, U2 = Infallible, U3 = Infallible> {
    T0(U0),
    T1(U1),
    T2(U2),
    T3(U3),
}
