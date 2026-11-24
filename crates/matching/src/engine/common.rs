use crate::protocol::{FollowUps, Instructions, MatchingProtocol, PlanError};
use crate::queue::{InputQueue, MatchingQueue, OutputQueue};
use crate::trigger::TriggerBooks;

use book::{BookError, level3::L3OrderBook};

use std::collections::VecDeque;

use thiserror::Error;

// ── Matching Error ────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MatchingError {
    #[error("planning failed: {0}")]
    Plan(#[from] PlanError),

    #[error("applying instructions failed: {0}")]
    ApplyFailed(BookError),
}

// ── Type Aliases ──────────────────────────────────────────────────────────────

pub type InputMessage<E> = <<E as MatchingEngine>::Input as MatchingQueue>::Message;
pub type OutputMessage<E> = <<E as MatchingEngine>::Output as MatchingQueue>::Message;

pub type EngineInstructions<E> = Instructions<
    <<E as MatchingEngine>::Book as L3OrderBook>::Incoming,
    <<E as MatchingEngine>::Triggers as TriggerBooks>::Update,
    OutputMessage<E>,
    InputMessage<E>,
>;

// ── Components ────────────────────────────────────────────────────────────────

pub struct Components<'a, E: MatchingEngine> {
    pub input: &'a mut E::Input,
    pub output: &'a mut E::Output,
    pub protocol: &'a mut E::Protocol,
    pub book: &'a mut E::Book,
    pub triggers: &'a mut E::Triggers,
}

// ── Matching Engine ───────────────────────────────────────────────────────────

pub trait MatchingEngine: Sized {
    type Input: InputQueue;
    type Output: OutputQueue;

    type Book: L3OrderBook;
    type Triggers: TriggerBooks;

    type Protocol: MatchingProtocol<
            Self::Book,
            Self::Triggers,
            Input = <Self::Input as MatchingQueue>::Message,
            Output = <Self::Output as MatchingQueue>::Message,
            Incoming = <Self::Book as L3OrderBook>::Incoming,
        >;

    fn components(&mut self) -> Components<'_, Self>;

    // ── Workflow ─────────────────────────────────────────────────────────────

    /// Processes the next input message, if any. Returns whether a message
    /// was processed.
    fn poll(&mut self) -> Result<bool, MatchingError> {
        let message = self.components().input.pop();

        match message {
            Some(message) => self.process(&message).map(|()| true),
            None => Ok(false),
        }
    }

    /// Processes a message, then its follow-ups in order until none are left.
    fn process(&mut self, message: &InputMessage<Self>) -> Result<(), MatchingError> {
        let mut pending = VecDeque::from_iter(self.step(message)?);

        while let Some(follow_up) = pending.pop_front() {
            pending.extend(self.step(&follow_up)?);
        }

        Ok(())
    }

    /// Plans and executes a single message, and returns its follow-ups.
    fn step(&mut self, message: &InputMessage<Self>) -> Result<FollowUps<InputMessage<Self>>, MatchingError> {
        let instructions = self.plan(message)?;

        self.execute(instructions)
    }

    /// Asks the protocol for the instructions of a message. The books are
    /// only borrowed immutably.
    fn plan(&mut self, message: &InputMessage<Self>) -> Result<EngineInstructions<Self>, PlanError> {
        let Components {
            protocol,
            book,
            triggers,
            ..
        } = self.components();

        protocol.plan(message, book, triggers)
    }

    /// Applies the updates to the books, pushes the outputs, and returns the
    /// follow-ups.
    fn execute(
        &mut self,
        instructions: EngineInstructions<Self>,
    ) -> Result<FollowUps<InputMessage<Self>>, MatchingError> {
        let Components {
            output, book, triggers, ..
        } = self.components();

        let Instructions {
            book_updates,
            trigger_updates,
            outputs,
            follow_ups,
        } = instructions;

        for update in book_updates {
            book.apply(update).map_err(MatchingError::ApplyFailed)?;
        }

        for update in trigger_updates {
            triggers.apply(update).map_err(MatchingError::ApplyFailed)?;
        }

        for message in outputs {
            output.push(message);
        }

        Ok(follow_ups)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ProtocolInstructions;
    use crate::trigger::TriggerUpdate;
    use book::{
        LocationError, MutationError,
        level3::{L3BookUpdate, L3BookView, VecL3OrderBook},
        trigger::{TriggerBookUpdate, TriggerBookView, TriggerCondition, VecTriggerBook},
    };
    use types::order::{
        LimitOrder, OrderSide, Price, TimeInForce,
        fields::OrderId,
        orders::{Order, RestingLimitOrder},
    };

    // ── Queues ───────────────────────────────────────────────────────────────

    impl<M> MatchingQueue for VecDeque<M> {
        type Message = M;
    }

    impl<M> InputQueue for VecDeque<M> {
        fn pop(&mut self) -> Option<M> {
            self.pop_front()
        }
    }

    impl<M> MatchingQueue for Vec<M> {
        type Message = M;
    }

    impl<M> OutputQueue for Vec<M> {
        fn push(&mut self, message: M) {
            Vec::push(self, message);
        }
    }

    // ── Protocol ─────────────────────────────────────────────────────────────

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Message {
        Submit(LimitOrder),
        Cancel(OrderId),
        Stop(Price, LimitOrder),
        Tick(Price),
        Amend(OrderId),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Event {
        Rested(OrderId),
        Cancelled(OrderId),
        Rejected(OrderId),
        Triggered(OrderId),
    }

    type TestBook = VecL3OrderBook<LimitOrder, RestingLimitOrder>;
    type Stops = (VecTriggerBook<Price, LimitOrder>,);

    /// Never trades: rests submitted orders, cancels known orders, and parks
    /// stop orders until a tick reaches their threshold.
    ///
    /// Deliberately sloppy to exercise the engine: duplicate ids are not
    /// checked (apply failure), and amending an unknown order is an error
    /// instead of a rejection (planning failure).
    struct Passive;

    impl<B: L3BookView> MatchingProtocol<B, Stops> for Passive {
        type Input = Message;
        type Output = Event;
        type Incoming = LimitOrder;

        fn plan(
            &mut self,
            message: &Message,
            book: &B,
            triggers: &Stops,
        ) -> Result<ProtocolInstructions<Self, B, Stops>, PlanError> {
            let mut instructions = Instructions::new();

            match *message {
                Message::Submit(order) => {
                    instructions.book_updates.push(L3BookUpdate::Insert(order));
                    instructions.outputs.push(Event::Rested(order.id()));
                }
                Message::Cancel(order_id) => match book.order(order_id) {
                    Some(_) => {
                        instructions.book_updates.push(L3BookUpdate::Remove(order_id));
                        instructions.outputs.push(Event::Cancelled(order_id));
                    }
                    None => instructions.outputs.push(Event::Rejected(order_id)),
                },
                Message::Stop(threshold, order) => {
                    let update = TriggerBookUpdate::Insert(threshold, order);
                    instructions.trigger_updates.push(TriggerUpdate::T0(update));
                }
                Message::Tick(price) => {
                    for (_, order) in triggers.0.orders_through(Some(&price)) {
                        let update = TriggerBookUpdate::Remove(order.id());
                        instructions.trigger_updates.push(TriggerUpdate::T0(update));
                        instructions.outputs.push(Event::Triggered(order.id()));
                        instructions.follow_ups.push(Message::Submit(*order));
                    }
                }
                Message::Amend(order_id) => {
                    book.order(order_id)
                        .ok_or(BookError::from(LocationError::IdNotFound { order_id }))?;
                }
            }

            Ok(instructions)
        }
    }

    // ── Engine ───────────────────────────────────────────────────────────────

    struct TestEngine {
        input: VecDeque<Message>,
        output: Vec<Event>,
        protocol: Passive,
        book: TestBook,
        triggers: Stops,
    }

    impl MatchingEngine for TestEngine {
        type Input = VecDeque<Message>;
        type Output = Vec<Event>;

        type Book = TestBook;
        type Triggers = Stops;

        type Protocol = Passive;

        fn components(&mut self) -> Components<'_, Self> {
            Components {
                input: &mut self.input,
                output: &mut self.output,
                protocol: &mut self.protocol,
                book: &mut self.book,
                triggers: &mut self.triggers,
            }
        }
    }

    fn limit(id: u64, side: OrderSide, price: u64) -> LimitOrder {
        LimitOrder::new(
            id.into(),
            5.into(),
            0.into(),
            side,
            Price::new(price),
            TimeInForce::GoodTillCanceled,
        )
    }

    fn make_engine(input: Vec<Message>) -> TestEngine {
        TestEngine {
            input: input.into(),
            output: Vec::new(),
            protocol: Passive,
            book: VecL3OrderBook::new(),
            triggers: (VecTriggerBook::new(TriggerCondition::AtOrAbove),),
        }
    }

    #[test]
    fn poll() {
        let mut engine = make_engine(vec![
            Message::Submit(limit(1, OrderSide::Bid, 100)),
            Message::Cancel(1.into()),
        ]);

        assert_eq!(engine.poll(), Ok(true));
        assert!(engine.book.order(1.into()).is_some());

        assert_eq!(engine.poll(), Ok(true));
        assert!(engine.book.order(1.into()).is_none());

        assert_eq!(engine.poll(), Ok(false));
        assert_eq!(engine.output, vec![Event::Rested(1.into()), Event::Cancelled(1.into())]);
    }

    #[test]
    fn process() {
        let mut engine = make_engine(Vec::new());

        assert_eq!(engine.process(&Message::Cancel(42.into())), Ok(()));
        assert_eq!(engine.output, vec![Event::Rejected(42.into())]);
    }

    #[test]
    fn process_follow_ups() {
        let mut engine = make_engine(Vec::new());
        let stop = limit(2, OrderSide::Bid, 106);

        assert_eq!(engine.process(&Message::Stop(Price::new(105), stop)), Ok(()));
        assert_eq!(engine.process(&Message::Tick(Price::new(104))), Ok(()));
        assert!(engine.output.is_empty());

        assert_eq!(engine.process(&Message::Tick(Price::new(105))), Ok(()));
        assert_eq!(engine.output, vec![Event::Triggered(2.into()), Event::Rested(2.into())]);
        assert!(engine.triggers.0.order(2.into()).is_none());
        assert!(engine.book.order(2.into()).is_some());
    }

    #[test]
    fn process_plan_failure() {
        let mut engine = make_engine(Vec::new());

        assert_eq!(
            engine.process(&Message::Amend(42.into())),
            Err(MatchingError::Plan(PlanError::BookError(BookError::LocationError(
                LocationError::IdNotFound { order_id: 42.into() }
            ))))
        );
        assert!(engine.output.is_empty());
    }

    #[test]
    fn process_apply_failure() {
        let mut engine = make_engine(Vec::new());
        let order = limit(1, OrderSide::Bid, 100);

        assert_eq!(engine.process(&Message::Submit(order)), Ok(()));
        assert_eq!(
            engine.process(&Message::Submit(order)),
            Err(MatchingError::ApplyFailed(BookError::MutationError(
                MutationError::IdAlreadyExists { order_id: 1.into() }
            )))
        );
        assert_eq!(engine.output, vec![Event::Rested(1.into())]);
    }

    #[test]
    fn plan() {
        let mut engine = make_engine(Vec::new());
        let order = limit(1, OrderSide::Bid, 100);

        let instructions = engine.plan(&Message::Submit(order)).unwrap();

        assert_eq!(instructions.book_updates.as_slice(), &[L3BookUpdate::Insert(order)]);
        assert_eq!(instructions.outputs.as_slice(), &[Event::Rested(1.into())]);
        assert!(engine.book.order(1.into()).is_none());
        assert!(engine.output.is_empty());
    }

    #[test]
    fn execute() {
        let mut engine = make_engine(Vec::new());
        let order = limit(1, OrderSide::Bid, 100);

        let mut instructions = Instructions::new();
        instructions.book_updates.push(L3BookUpdate::Insert(order));
        instructions.outputs.push(Event::Rested(1.into()));
        instructions.follow_ups.push(Message::Cancel(1.into()));

        let follow_ups = engine.execute(instructions).unwrap();

        assert_eq!(follow_ups.as_slice(), &[Message::Cancel(1.into())]);
        assert!(engine.book.order(1.into()).is_some());
        assert_eq!(engine.output, vec![Event::Rested(1.into())]);
    }
}
