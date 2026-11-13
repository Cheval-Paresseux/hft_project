use crate::{
    common::MatchingError,
    controller::BookController,
    event::{MarketEvent, MatchingEvent},
    strategy::MatchingStrategy,
};

// ── Matching Engine ───────────────────────────────────────────────────────────

pub struct MatchingEngine<C: BookController, S: MatchingStrategy<C::Book>> {
    controller: C,
    strategy: S,
}

impl<C: BookController, S: MatchingStrategy<C::Book>> MatchingEngine<C, S> {
    pub fn new(controller: C, strategy: S) -> Self {
        Self { controller, strategy }
    }

    pub fn controller(&self) -> &C {
        &self.controller
    }

    pub fn on_event(&mut self, event: MarketEvent) -> Result<Vec<MatchingEvent>, EngineError> {
        // 1. Peek → plan. The view borrow dies with this block, so…
        let plan = {
            let book = self.controller.view();
            self.strategy.plan(&event, book).map_err(EngineError::Strategy)?
        };

        // 2. Split into the two message kinds.
        let (batch, events) = plan.into_parts();

        // 3. Apply — …the exclusive borrow happens here. The plan/apply split
        //    *is* the borrow-checker story: this line failing to compile would
        //    mean the design is broken.
        self.controller.execute(batch).map_err(EngineError::Controller)?;

        // 4. Emit trading semantics downstream.
        Ok(events.into_vec())
    }
}
