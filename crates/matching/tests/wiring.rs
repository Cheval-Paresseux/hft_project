use matching::common::MatchingError;
use matching::controller::{BookController, BookInstruction, L3Controller};
use matching::engine::{EngineError, MatchingEngine};
use matching::event::{Fill, MarketEvent, MatchingEvent};
use matching::plan::Plan;
use matching::strategy::{AllocationAlgorithm, ExecutionMode, MatchingStrategy, Strategy};

use book::level3::{L3OrderBook, VecL3OrderBook};
use book::OrderMutationsPolicy;

use types::order::{
    fields::{OrderId, OrderSide, Price, Quantity},
    orders::{IncomingOrder, Order, RestingLimitOrder},
    LimitOrder, TimeInForce,
};

use smallvec::SmallVec;

// ── Book Fixture ──────────────────────────────────────────────────────────────

type TestBook = VecL3OrderBook<LimitOrder, RestingLimitOrder>;

fn make_book() -> TestBook {
    VecL3OrderBook::new(OrderMutationsPolicy::KeepPriority, OrderMutationsPolicy::KeepPriority)
}

fn limit_order(id: u64, quantity: u64, side: OrderSide, price: u64) -> LimitOrder {
    LimitOrder::new(
        id.into(),
        quantity.into(),
        0.into(),
        side,
        Price::new(price),
        TimeInForce::GoodTillCanceled,
    )
}

// ── Test Stubs ────────────────────────────────────────────────────────────────
// Toy implementations — they exist to validate that the pieces interact, not
// to match orders correctly.

/// Fills the best order, up to `remaining`.
struct TestAllocation;

impl AllocationAlgorithm for TestAllocation {
    fn allocate(
        &self,
        mut orders: impl Iterator<Item = (OrderId, Quantity)>,
        remaining: Quantity,
        fills: &mut SmallVec<[Fill; 4]>,
    ) {
        if let Some((order_id, quantity)) = orders.next() {
            let filled = if quantity < remaining { quantity } else { remaining };

            if !filled.is_zero() {
                fills.push(Fill {
                    order_id,
                    quantity: filled,
                });
            }
        }
    }
}

/// Minimal workflow: allocate against the opposite side, consume the makers,
/// rest the remainder of the incoming order.
struct TestExecution;

impl<B> ExecutionMode<B> for TestExecution
where
    B: L3OrderBook<Incoming = LimitOrder>,
{
    fn plan<A: AllocationAlgorithm>(
        &mut self,
        event: &MarketEvent,
        book: &B,
        allocation: &A,
        plan: &mut Plan<B::Incoming>,
    ) -> Result<(), MatchingError> {
        let MarketEvent::New(order) = event else {
            return Ok(());
        };

        let mut fills = SmallVec::<[Fill; 4]>::new();
        allocation.allocate(
            book.orders_through(order.side().opposite(), Some(order.limit()))
                .map(|(_, _, resting)| (resting.id(), resting.quantity())),
            order.quantity(),
            &mut fills,
        );

        let mut filled = 0;
        for fill in &fills {
            let (_, price, resting) = book
                .order(fill.order_id)
                .expect("allocation returned an unknown order");
            let remaining = resting.quantity().get() - fill.quantity.get();

            plan.push_instruction(BookInstruction::Reduce(fill.order_id, Quantity::new(remaining)));
            plan.push_event(MatchingEvent::Executed {
                maker: fill.order_id,
                taker: order.id(),
                price,
                quantity: fill.quantity,
            });

            filled += fill.quantity.get();
        }

        let remainder = order.quantity().get() - filled;
        if remainder > 0 {
            let resting = LimitOrder::new(
                order.id(),
                Quantity::new(remainder),
                order.timestamp(),
                order.side(),
                order.limit(),
                order.tif(),
            );

            plan.push_instruction(BookInstruction::Insert(resting));
            plan.push_event(MatchingEvent::Resting {
                order_id: order.id(),
            });
        }

        Ok(())
    }
}

/// Always fails — validates that a planner failure cancels the event.
struct FailingStrategy;

impl MatchingStrategy<TestBook> for FailingStrategy {
    type Execution = TestExecution;
    type Allocation = TestAllocation;

    fn plan(
        &mut self,
        _event: &MarketEvent,
        _book: &TestBook,
    ) -> Result<Plan<<TestBook as L3OrderBook>::Incoming>, MatchingError> {
        Err(MatchingError::BookError(
            book::BookError::OrderIdAlreadyExists { order_id: 1.into() },
        ))
    }
}

fn make_engine() -> MatchingEngine<L3Controller<TestBook>, Strategy<TestExecution, TestAllocation>> {
    MatchingEngine::new(
        L3Controller::new(make_book()),
        Strategy::new(TestExecution, TestAllocation),
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn strategy_produces_plan_from_book_view() {
    let book = make_book();
    let mut strategy = Strategy::new(TestExecution, TestAllocation);

    let plan = strategy
        .plan(&MarketEvent::New(limit_order(1, 4, OrderSide::Bid, 100)), &book)
        .expect("planning against an empty book should succeed");

    assert_eq!(plan.events(), &[MatchingEvent::Resting { order_id: 1.into() }]);

    let instructions: Vec<_> = plan.instructions().iter().collect();
    assert!(matches!(instructions.as_slice(), [BookInstruction::Insert(_)]));
}

#[test]
fn engine_round_trip_rests_the_order_in_the_book() {
    let mut engine = make_engine();

    let events = engine
        .on_event(MarketEvent::New(limit_order(1, 4, OrderSide::Bid, 100)))
        .expect("event should be planned and applied");

    assert_eq!(events, [MatchingEvent::Resting { order_id: 1.into() }]);

    let book = engine.controller().view();
    let resting = book.order(1.into()).expect("order should rest in the book");

    assert_eq!(resting.1, Price::new(100));
    assert_eq!(resting.2.quantity(), Quantity::new(4));
    assert_eq!(book.quantity_at(OrderSide::Bid, Price::new(100)), Quantity::new(4));
}

#[test]
fn engine_round_trip_executes_against_liquidity() {
    let mut book = make_book();
    book.insert(limit_order(1, 10, OrderSide::Ask, 100)).unwrap();

    let mut engine = MatchingEngine::new(
        L3Controller::new(book),
        Strategy::new(TestExecution, TestAllocation),
    );

    let events = engine
        .on_event(MarketEvent::New(limit_order(2, 4, OrderSide::Bid, 100)))
        .expect("event should be planned and applied");

    assert_eq!(
        events,
        [MatchingEvent::Executed {
            maker: 1.into(),
            taker: 2.into(),
            price: Price::new(100),
            quantity: Quantity::new(4),
        }]
    );

    let book = engine.controller().view();
    let maker = book.order(1.into()).expect("maker should remain in the book");

    assert_eq!(maker.2.quantity(), Quantity::new(6));
    assert_eq!(book.quantity_at(OrderSide::Ask, Price::new(100)), Quantity::new(6));
    assert!(book.order(2.into()).is_none(), "incoming order was fully filled");
}

#[test]
fn planner_failure_cancels_the_event_without_mutation() {
    let mut engine = MatchingEngine::new(L3Controller::new(make_book()), FailingStrategy);

    let error = engine
        .on_event(MarketEvent::New(limit_order(1, 4, OrderSide::Bid, 100)))
        .expect_err("a failing strategy must cancel the event");

    assert_eq!(
        error,
        EngineError::Strategy(MatchingError::BookError(
            book::BookError::OrderIdAlreadyExists { order_id: 1.into() }
        ))
    );
    assert!(engine.controller().view().order(1.into()).is_none());
}

#[test]
fn controller_failure_reports_the_failing_stage() {
    let mut book = make_book();
    book.insert(limit_order(1, 10, OrderSide::Bid, 100)).unwrap();

    let mut engine = MatchingEngine::new(
        L3Controller::new(book),
        Strategy::new(TestExecution, TestAllocation),
    );

    // Duplicate order id: planning succeeds, applying the insert fails.
    let error = engine
        .on_event(MarketEvent::New(limit_order(1, 4, OrderSide::Bid, 105)))
        .expect_err("a duplicate id must fail during application");

    assert_eq!(
        error,
        EngineError::Controller(MatchingError::BookError(
            book::BookError::OrderIdAlreadyExists { order_id: 1.into() }
        ))
    );

    let book = engine.controller().view();
    assert_eq!(book.quantity_at(OrderSide::Bid, Price::new(105)), Quantity::new(0));
    assert_eq!(book.quantity_at(OrderSide::Bid, Price::new(100)), Quantity::new(10));
}
