use crate::common::MatchingError;
use crate::event::{Fill, MarketEvent};
use crate::plan::Plan;

use book::level3::L3OrderBook;

use types::order::fields::{OrderId, Quantity};

use smallvec::SmallVec;

// ── Execution Mode ────────────────────────────────────────────────────────────

/// The "when" workflow: continuous matching, call auction, …
///
/// Outer trait — it drives the allocation rule while building the plan, and
/// decides which events are interesting (an empty plan is a valid answer).
pub trait ExecutionMode<B: L3OrderBook> {
    fn plan<A: AllocationAlgorithm>(
        &mut self,
        event: &MarketEvent,
        book: &B,
        allocation: &A,
        plan: &mut Plan<B::Incoming>,
    ) -> Result<(), MatchingError>;
}

// ── Allocation Algorithm ──────────────────────────────────────────────────────

/// The "whom" rule: price-time, pro-rata, …
///
/// Purely combinatorial — it sees order ids and quantities, never instructions
/// or events; the execution mode turns the produced fills into both.
pub trait AllocationAlgorithm {
    fn allocate(
        &self,
        orders: impl Iterator<Item = (OrderId, Quantity)>,
        remaining: Quantity,
        fills: &mut SmallVec<[Fill; 4]>,
    );
}

// ── Matching Strategy ─────────────────────────────────────────────────────────

/// The planner: peeks at the book (read-only) and produces the plan for an
/// incoming event. Failing here cancels the event before any mutation.
pub trait MatchingStrategy<B: L3OrderBook> {
    type Execution: ExecutionMode<B>;
    type Allocation: AllocationAlgorithm;

    fn plan(&mut self, event: &MarketEvent, book: &B) -> Result<Plan<B::Incoming>, MatchingError>;
}

// ── Strategy Combinator ───────────────────────────────────────────────────────

/// Pairs an execution mode with an allocation rule, so every combination of
/// the two axes is one struct literal away.
pub struct Strategy<E, A> {
    pub execution: E,
    pub allocation: A,
}

impl<E, A> Strategy<E, A> {
    pub fn new(execution: E, allocation: A) -> Self {
        Self { execution, allocation }
    }
}

impl<B, E, A> MatchingStrategy<B> for Strategy<E, A>
where
    B: L3OrderBook,
    E: ExecutionMode<B>,
    A: AllocationAlgorithm,
{
    type Execution = E;
    type Allocation = A;

    fn plan(&mut self, event: &MarketEvent, book: &B) -> Result<Plan<B::Incoming>, MatchingError> {
        let mut plan = Plan::new();

        self.execution.plan(event, book, &self.allocation, &mut plan)?;

        Ok(plan)
    }
}
