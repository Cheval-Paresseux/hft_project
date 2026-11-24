use super::common::{TriggerBookUpdateOf, TriggerBooks, TriggerUpdate};

use book::{BookError, trigger::TriggerBook};

use std::convert::Infallible;

// ── No Trigger Books ──────────────────────────────────────────────────────────

impl TriggerBooks for () {
    type Update = Infallible;

    fn apply(&mut self, update: Infallible) -> Result<(), BookError> {
        match update {}
    }
}

// ── Tuples of Trigger Books ───────────────────────────────────────────────────

impl<A: TriggerBook> TriggerBooks for (A,) {
    type Update = TriggerUpdate<TriggerBookUpdateOf<A>>;

    fn apply(&mut self, update: Self::Update) -> Result<(), BookError> {
        match update {
            TriggerUpdate::T0(update) => self.0.apply(update),
        }
    }
}

impl<A: TriggerBook, B: TriggerBook> TriggerBooks for (A, B) {
    type Update = TriggerUpdate<TriggerBookUpdateOf<A>, TriggerBookUpdateOf<B>>;

    fn apply(&mut self, update: Self::Update) -> Result<(), BookError> {
        match update {
            TriggerUpdate::T0(update) => self.0.apply(update),
            TriggerUpdate::T1(update) => self.1.apply(update),
        }
    }
}

impl<A: TriggerBook, B: TriggerBook, C: TriggerBook> TriggerBooks for (A, B, C) {
    type Update = TriggerUpdate<TriggerBookUpdateOf<A>, TriggerBookUpdateOf<B>, TriggerBookUpdateOf<C>>;

    fn apply(&mut self, update: Self::Update) -> Result<(), BookError> {
        match update {
            TriggerUpdate::T0(update) => self.0.apply(update),
            TriggerUpdate::T1(update) => self.1.apply(update),
            TriggerUpdate::T2(update) => self.2.apply(update),
        }
    }
}

impl<A: TriggerBook, B: TriggerBook, C: TriggerBook, D: TriggerBook> TriggerBooks for (A, B, C, D) {
    type Update =
        TriggerUpdate<TriggerBookUpdateOf<A>, TriggerBookUpdateOf<B>, TriggerBookUpdateOf<C>, TriggerBookUpdateOf<D>>;

    fn apply(&mut self, update: Self::Update) -> Result<(), BookError> {
        match update {
            TriggerUpdate::T0(update) => self.0.apply(update),
            TriggerUpdate::T1(update) => self.1.apply(update),
            TriggerUpdate::T2(update) => self.2.apply(update),
            TriggerUpdate::T3(update) => self.3.apply(update),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use book::{
        LocationError,
        trigger::{TriggerBookUpdate, TriggerBookView, TriggerCondition, VecTriggerBook},
    };
    use types::order::{
        fields::{Price, Timestamp},
        orders::RestingLimitOrder,
    };

    type Stops = VecTriggerBook<Price, RestingLimitOrder>;
    type Timers = VecTriggerBook<Timestamp, RestingLimitOrder>;

    fn make_order(id: u64) -> RestingLimitOrder {
        RestingLimitOrder::new(id.into(), 5.into(), 0.into())
    }

    fn make_books() -> (Stops, Timers) {
        (
            VecTriggerBook::new(TriggerCondition::AtOrAbove),
            VecTriggerBook::new(TriggerCondition::AtOrBelow),
        )
    }

    #[test]
    fn apply() {
        let mut books = make_books();

        let stop = TriggerUpdate::T0(TriggerBookUpdate::Insert(Price::new(100), make_order(1)));
        let timer = TriggerUpdate::T1(TriggerBookUpdate::Insert(Timestamp::new(10), make_order(2)));

        assert_eq!(books.apply(stop), Ok(()));
        assert_eq!(books.apply(timer), Ok(()));

        assert!(books.0.order(1.into()).is_some());
        assert!(books.1.order(2.into()).is_some());
        assert!(books.0.order(2.into()).is_none());
    }

    #[test]
    fn apply_failure() {
        let mut books = make_books();

        assert_eq!(
            books.apply(TriggerUpdate::T1(TriggerBookUpdate::Remove(42.into()))),
            Err(BookError::LocationError(LocationError::IdNotFound {
                order_id: 42.into()
            }))
        );
    }
}
