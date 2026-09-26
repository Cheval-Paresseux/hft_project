use super::{
    execution::{ExecutionInstruction, ExecutionPolicy},
    message::EngineMessage,
};

use crate::order::{
    LimitOrder, MarketOrder, MarketTimeInForce, Order, StopLimitOrder, StopOrder, TimeInForce,
};

use orderbook::{
    errors::OrderBookError,
    level3::L3OrderBook,
    order::{LimitOrder as BookLimitOrder, OrderId, OrderSide, Price, Quantity},
};

// ── Matching Engine ───────────────────────────────────────────────────────────

pub struct L3MatchingEngine<OB: L3OrderBook> {
    orderbook: OB,
}

impl<OB: L3OrderBook> Default for L3MatchingEngine<OB> {
    fn default() -> Self {
        Self {
            orderbook: OB::default(),
        }
    }
}

// ── Message and Execution ─────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    pub fn submit_message(&mut self, message: EngineMessage) {
        let policy = match message {
            EngineMessage::NewOrder(order) => self.new_order_policy(order),
            EngineMessage::CancelOrder { order_id } => self.cancel_order_policy(order_id),
            EngineMessage::ModifyOrder {
                order_id,
                new_quantity,
            } => self.modify_order_policy(order_id, new_quantity),
        };

        self.execute(policy);
    }

    fn execute(&mut self, policy: ExecutionPolicy) {
        for instruction in policy.instructions() {
            let result = match instruction {
                ExecutionInstruction::Add(order) => {
                    self.orderbook.add_order(*order);
                    Ok(())
                }
                ExecutionInstruction::Cancel { order_id } => self.orderbook.cancel_order(*order_id),
                ExecutionInstruction::Modify {
                    order_id,
                    new_quantity,
                } => self.orderbook.modify_order(*order_id, *new_quantity),
                ExecutionInstruction::Fill {
                    order_id,
                    fill_quantity,
                } => self.orderbook.fill_order(*order_id, *fill_quantity),
            };
        }
        //todo: what do we do with the Result ?
    }
}

// ── Execution Policies ────────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    fn new_order_policy(&self, order: Order) -> ExecutionPolicy {
        match order {
            Order::Market(o) => self.market_order_policy(o),
            Order::Limit(o) => self.limit_order_policy(o),

            Order::Stop(_) | Order::StopLimit(_) => ExecutionPolicy::new(),
        }
    }

    fn cancel_order_policy(&self, order_id: OrderId) -> ExecutionPolicy {
        let mut policy = ExecutionPolicy::with_capacity(1);
        policy.add(ExecutionInstruction::Cancel { order_id });

        policy
    }

    fn modify_order_policy(&self, order_id: OrderId, new_quantity: Quantity) -> ExecutionPolicy {
        let mut policy = ExecutionPolicy::with_capacity(1);
        policy.add(ExecutionInstruction::Modify {
            order_id,
            new_quantity,
        });

        policy
    }
}

// ── Matching ───────────────────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    fn match_against_book(
        &self,
        side: OrderSide,
        price_bound: Option<Price>,
        mut remaining: Quantity,
    ) -> (ExecutionPolicy, Quantity) {
        let mut policy = ExecutionPolicy::new();

        for (order_id, _, resting) in self.orderbook.orders_up_to(side.opposite(), price_bound) {
            if remaining.is_zero() {
                break;
            }

            let fill = if resting <= remaining {
                resting
            } else {
                remaining
            };

            policy.add(ExecutionInstruction::Fill {
                order_id,
                fill_quantity: fill,
            });
            remaining -= fill;
        }

        (policy, remaining)
    }

    fn limit_order_policy(&self, limit: LimitOrder) -> ExecutionPolicy {
        if limit.tif == TimeInForce::FillOrKill {
            let available = self
                .orderbook
                .quantity_up_to(limit.side.opposite(), Some(limit.price));

            if available < limit.quantity {
                return ExecutionPolicy::new();
            }
        }

        let (mut policy, remaining) =
            self.match_against_book(limit.side, Some(limit.price), limit.quantity);

        let rests = matches!(limit.tif, TimeInForce::Day | TimeInForce::GoodTillCanceled);
        if rests && !remaining.is_zero() {
            policy.add(ExecutionInstruction::Add(limit.into_book(remaining)));
        }

        policy
    }

    fn market_order_policy(&self, market: MarketOrder) -> ExecutionPolicy {
        if market.tif == MarketTimeInForce::FillOrKill {
            let available = self.orderbook.quantity_up_to(market.side.opposite(), None);

            if available < market.quantity {
                return ExecutionPolicy::new();
            }
        }

        let (policy, _unfilled) = self.match_against_book(market.side, None, market.quantity);

        policy
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use orderbook::{level3::VecL3OrderBook, order::Timestamp};

    type Engine = L3MatchingEngine<VecL3OrderBook>;

    fn engine_with(resting: &[BookLimitOrder]) -> Engine {
        let mut engine = Engine::default();

        for order in resting {
            engine.orderbook.add_order(*order);
        }

        engine
    }

    fn book_limit_order(id: u64, quantity: u64, side: OrderSide, price: u64) -> BookLimitOrder {
        let timestamp = Timestamp::new(0);
        BookLimitOrder::new(id.into(), timestamp, quantity.into(), side, price.into())
    }

    fn limit_order(
        id: u64,
        quantity: u64,
        side: OrderSide,
        price: u64,
        tif: TimeInForce,
    ) -> LimitOrder {
        let timestamp = Timestamp::new(0);
        LimitOrder::new(
            id.into(),
            timestamp,
            quantity.into(),
            side,
            price.into(),
            tif,
        )
    }

    fn market_order(
        id: u64,
        quantity: u64,
        side: OrderSide,
        tif: MarketTimeInForce,
    ) -> MarketOrder {
        let timestamp = Timestamp::new(0);
        MarketOrder::new(id.into(), timestamp, quantity.into(), side, tif)
    }

    fn fill_instruction(order_id: u64, quantity: u64) -> ExecutionInstruction {
        ExecutionInstruction::Fill {
            order_id: order_id.into(),
            fill_quantity: quantity.into(),
        }
    }

    fn add_instruction(limit: &LimitOrder, resting: u64) -> ExecutionInstruction {
        ExecutionInstruction::Add(limit.into_book(resting.into()))
    }

    #[test]
    fn new_order_policy() {
        let side = OrderSide::Bid;
        let engine = engine_with(&[
            book_limit_order(0, 3, side, 100),
            book_limit_order(1, 3, side, 101),
        ]);

        let limit_order = limit_order(9, 3, side, 100, TimeInForce::GoodTillCanceled);
        let policy = engine.new_order_policy(Order::Limit(limit_order));

        assert_eq!(
            policy.instructions(),
            &[fill_instruction(1, 3), add_instruction(&limit_order, 2)]
        );

        let market_order = market_order(9, 3, side, MarketTimeInForce::ImmediateOrCancel);
        let policy = engine.new_order_policy(Order::Market(market_order));

        assert_eq!(
            policy.instructions(),
            &[fill_instruction(1, 3), fill_instruction(2, 2)]
        );
    }

    #[test]
    fn match_against_book() {
        let engine = engine_with(&[
            make_limit(1, 2, OrderSide::Ask, 101),
            make_limit(2, 2, OrderSide::Ask, 100),
            make_limit(3, 2, OrderSide::Ask, 100),
            make_limit(4, 2, OrderSide::Bid, 100),
        ]);

        // An incoming bid only ever sees the ask side, and the bound is inclusive,
        // so the 101 level is out of reach. Order 4 is a bid and never a candidate.
        let (policy, remaining) =
            engine.match_against_book(OrderSide::Bid, Some(100.into()), 10.into());

        assert_eq!(policy.instructions(), &[fill(2, 2), fill(3, 2)]);
        assert_eq!(remaining, 6.into());

        // The second level is only partially consumed.
        let (policy, remaining) =
            engine.match_against_book(OrderSide::Bid, Some(100.into()), 3.into());

        assert_eq!(policy.instructions(), &[fill(2, 2), fill(3, 1)]);
        assert_eq!(remaining, 0.into());

        // A market order has no bound, so it also reaches the 101 level.
        let (policy, remaining) = engine.match_against_book(OrderSide::Bid, None, 8.into());

        assert_eq!(policy.instructions(), &[fill(2, 2), fill(3, 2), fill(1, 2)]);
        assert_eq!(remaining, 2.into());

        // An incoming ask walks the bid side instead.
        let (policy, remaining) = engine.match_against_book(OrderSide::Ask, None, 8.into());

        assert_eq!(policy.instructions(), &[fill(4, 2)]);
        assert_eq!(remaining, 6.into());

        // A bound that no resting price reaches leaves the quantity untouched.
        let (policy, remaining) =
            engine.match_against_book(OrderSide::Bid, Some(99.into()), 5.into());

        assert!(policy.is_empty());
        assert_eq!(remaining, 5.into());
    }

    #[test]
    fn limit_order_policy() {
        let engine = engine_with(&[
            make_limit(1, 5, OrderSide::Ask, 100),
            make_limit(2, 5, OrderSide::Ask, 101),
        ]);

        // 13 wants in, 10 reachable, so the remainder rests. The Add comes last.
        for tif in [TimeInForce::GoodTillCanceled, TimeInForce::Day] {
            let limit = incoming(9, 13, OrderSide::Bid, 101, tif);

            assert_eq!(
                engine.limit_order_policy(limit).instructions(),
                &[fill(1, 5), fill(2, 5), added(&limit, 3)]
            );
        }

        // ImmediateOrCancel fills the same 10 and drops the remainder.
        assert_eq!(
            engine
                .limit_order_policy(incoming(
                    9,
                    13,
                    OrderSide::Bid,
                    101,
                    TimeInForce::ImmediateOrCancel
                ))
                .instructions(),
            &[fill(1, 5), fill(2, 5)]
        );

        // FillOrKill matches 8 out of the 10 reachable, and rests nothing.
        assert_eq!(
            engine
                .limit_order_policy(incoming(9, 8, OrderSide::Bid, 101, TimeInForce::FillOrKill))
                .instructions(),
            &[fill(1, 5), fill(2, 3)]
        );

        // ...but gives up entirely when 13 are asked for and only 10 exist.
        assert!(engine
            .limit_order_policy(incoming(
                9,
                13,
                OrderSide::Bid,
                101,
                TimeInForce::FillOrKill
            ))
            .is_empty());

        // A sell crosses the best bid first and stops at its own price.
        let engine = engine_with(&[
            make_limit(1, 2, OrderSide::Bid, 100),
            make_limit(2, 3, OrderSide::Bid, 99),
        ]);

        assert_eq!(
            engine
                .limit_order_policy(incoming(9, 4, OrderSide::Ask, 99, TimeInForce::Day))
                .instructions(),
            &[fill(1, 2), fill(2, 2)]
        );

        // Nothing rests when the price does not cross.
        let engine = engine_with(&[make_limit(1, 5, OrderSide::Ask, 101)]);

        let limit = incoming(9, 5, OrderSide::Bid, 100, TimeInForce::GoodTillCanceled);

        assert_eq!(
            engine.limit_order_policy(limit).instructions(),
            &[added(&limit, 5)]
        );
    }

    #[test]
    fn market_order_policy() {
        let engine = engine_with(&[
            make_limit(1, 3, OrderSide::Ask, 100),
            make_limit(2, 3, OrderSide::Ask, 101),
        ]);

        // Sweeps every reachable level, then drops the remainder.
        assert_eq!(
            engine
                .market_order_policy(market(
                    9,
                    4,
                    OrderSide::Bid,
                    MarketTimeInForce::ImmediateOrCancel
                ))
                .instructions(),
            &[fill(1, 3), fill(2, 1)]
        );

        // A sell sweeps the bid side from the top down.
        let engine = engine_with(&[
            make_limit(1, 3, OrderSide::Bid, 100),
            make_limit(2, 3, OrderSide::Bid, 99),
        ]);

        assert_eq!(
            engine
                .market_order_policy(market(9, 4, OrderSide::Ask, MarketTimeInForce::FillOrKill))
                .instructions(),
            &[fill(1, 3), fill(2, 1)]
        );

        // FillOrKill gives up entirely when 10 are asked for and only 6 exist.
        assert!(engine
            .market_order_policy(market(9, 10, OrderSide::Ask, MarketTimeInForce::FillOrKill))
            .is_empty());

        // An empty book has nothing to sweep, whatever the time in force.
        assert!(Engine::default()
            .market_order_policy(market(
                9,
                4,
                OrderSide::Bid,
                MarketTimeInForce::ImmediateOrCancel
            ))
            .is_empty());
    }

    #[test]
    fn submit_message() {
        let mut engine = engine_with(&[
            make_limit(1, 5, OrderSide::Ask, 100),
            make_limit(2, 4, OrderSide::Ask, 101),
        ]);

        // A crossing order fills both levels and rests the tenth unit.
        engine.submit_message(EngineMessage::NewOrder(Order::Limit(incoming(
            9,
            10,
            OrderSide::Bid,
            101,
            TimeInForce::GoodTillCanceled,
        ))));

        assert_eq!(
            engine.orderbook.quantity_at(OrderSide::Ask, 100.into()),
            0.into()
        );
        assert_eq!(
            engine.orderbook.quantity_at(OrderSide::Ask, 101.into()),
            0.into()
        );
        assert_eq!(
            engine.orderbook.quantity_at(OrderSide::Bid, 101.into()),
            1.into()
        );

        // Cancel clears the resting remainder.
        engine.submit_message(EngineMessage::CancelOrder { order_id: 9.into() });

        assert_eq!(
            engine.orderbook.quantity_at(OrderSide::Bid, 101.into()),
            0.into()
        );

        // Modify leaves the order resting at its original price.
        engine.submit_message(EngineMessage::NewOrder(Order::Limit(incoming(
            9,
            3,
            OrderSide::Bid,
            99,
            TimeInForce::GoodTillCanceled,
        ))));

        engine.submit_message(EngineMessage::ModifyOrder {
            order_id: 9.into(),
            new_quantity: 8.into(),
        });

        assert_eq!(
            engine.orderbook.quantity_at(OrderSide::Bid, 99.into()),
            8.into()
        );

        // A resting order must not cross the ask it creates in the same policy.
        let mut engine = engine_with(&[]);

        engine.submit_message(EngineMessage::NewOrder(Order::Limit(incoming(
            9,
            5,
            OrderSide::Bid,
            100,
            TimeInForce::GoodTillCanceled,
        ))));

        assert_eq!(
            engine.orderbook.quantity_at(OrderSide::Bid, 100.into()),
            5.into()
        );
        assert_eq!(
            engine.orderbook.quantity_at(OrderSide::Ask, 100.into()),
            0.into()
        );
    }
}
