use super::{
    execution::{ExecutionInstruction, ExecutionPolicy},
    message::EngineMessage
};

use crate::order::{
    Order, MarketOrder, LimitOrder, StopOrder, StopLimitOrder,
    MarketTimeInForce, TimeInForce,
};

use orderbook::{
    errors::OrderBookError,
    level3::L3OrderBook,
    order::{LimitOrder as BookLimitOrder, OrderId, OrderSide, Price, Quantity}
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
            EngineMessage::CancelOrder{ order_id } => self.cancel_order_policy(order_id),
            EngineMessage::ModifyOrder{ order_id, new_quantity } => self.modify_order_policy(order_id, new_quantity), 
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
                ExecutionInstruction::Cancel { order_id } => {
                    self.orderbook.cancel_order(*order_id)
                }
                ExecutionInstruction::Modify { order_id, new_quantity } => {
                    self.orderbook.modify_order(*order_id, *new_quantity)
                }
                ExecutionInstruction::Fill {order_id, fill_quantity } => {
                    self.orderbook.fill_order(*order_id, *fill_quantity)
                }
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
        policy.add(ExecutionInstruction::Modify { order_id, new_quantity });

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

            let fill = if resting <= remaining { resting } else { remaining };

            policy.add(ExecutionInstruction::Fill { order_id, fill_quantity: fill });
            remaining -= fill;
        }

        (policy, remaining)
    }

    fn limit_order_policy(&self, limit: LimitOrder) -> ExecutionPolicy {
        if limit.tif == TimeInForce::FillOrKill {
            let available = self.orderbook.quantity_up_to(limit.side.opposite(), Some(limit.price));

            if available < limit.quantity {
                return ExecutionPolicy::new();
            }
        }

        let (mut policy, remaining) = self.match_against_book(limit.side, Some(limit.price), limit.quantity);

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

    fn resting(id: u64, quantity: u64, side: OrderSide, price: u64) -> BookLimitOrder {
        BookLimitOrder::new(
            id.into(),
            Timestamp::new(0),
            quantity.into(),
            side,
            price.into(),
        )
    }

    fn engine_with(resting_orders: &[BookLimitOrder]) -> Engine {
        let mut engine = Engine::default();

        for order in resting_orders {
            engine.orderbook.add_order(*order);
        }

        engine
    }

    fn limit(
        id: u64,
        quantity: u64,
        side: OrderSide,
        price: u64,
        tif: TimeInForce,
    ) -> Order {
        Order::Limit(LimitOrder::new(
            id.into(),
            Timestamp::new(0),
            quantity.into(),
            side,
            price.into(),
            tif,
        ))
    }

    fn market(id: u64, quantity: u64, side: OrderSide, tif: MarketTimeInForce) -> Order {
        Order::Market(MarketOrder::new(
            id.into(),
            Timestamp::new(0),
            quantity.into(),
            side,
            tif,
        ))
    }

    fn fill(order_id: u64, quantity: u64) -> ExecutionInstruction {
        ExecutionInstruction::Fill {
            order_id: order_id.into(),
            fill_quantity: quantity.into(),
        }
    }

    // ── Limit Orders ──────────────────────────────────────────────────────────────

    #[test]
    fn limit_bid_crosses_ask_and_rests_nothing_when_fully_filled() {
        let engine = engine_with(&[resting(1, 5, OrderSide::Ask, 100)]);

        let policy = engine.limit_order_policy(LimitOrder::new(
            2.into(),
            Timestamp::new(0),
            5.into(),
            OrderSide::Bid,
            100.into(),
            TimeInForce::GoodTillCanceled,
        ));

        assert_eq!(policy.instructions(), &[fill(1, 5)]);
    }

    #[test]
    fn limit_bid_rests_the_remainder_after_a_partial_fill() {
        let engine = engine_with(&[resting(1, 3, OrderSide::Ask, 100)]);

        let policy = engine.limit_order_policy(LimitOrder::new(
            2.into(),
            Timestamp::new(0),
            8.into(),
            OrderSide::Bid,
            100.into(),
            TimeInForce::GoodTillCanceled,
        ));

        let add = ExecutionInstruction::Add(
            LimitOrder::new(
                2.into(),
                Timestamp::new(0),
                8.into(),
                OrderSide::Bid,
                100.into(),
                TimeInForce::GoodTillCanceled,
            )
            .into_book(5.into()),
        );

        // Fill first, then rest, so the order can never match against itself.
        assert_eq!(policy.instructions(), &[fill(1, 3), add]);
    }

    #[test]
    fn limit_bid_sweeps_asks_in_price_then_time_priority() {
        let engine = engine_with(&[
            resting(1, 2, OrderSide::Ask, 101),
            resting(2, 2, OrderSide::Ask, 100),
            resting(3, 2, OrderSide::Ask, 100),
            resting(4, 2, OrderSide::Ask, 102),
        ]);

        let policy = engine.limit_order_policy(LimitOrder::new(
            9.into(),
            Timestamp::new(0),
            5.into(),
            OrderSide::Bid,
            101.into(),
            TimeInForce::GoodTillCanceled,
        ));

        // 100 level in insertion order (2 then 3), then 101, and 102 is out of reach.
        assert_eq!(policy.instructions(), &[fill(2, 2), fill(3, 2), fill(1, 1)]);
    }

    #[test]
    fn limit_does_not_cross_beyond_its_price() {
        let engine = engine_with(&[resting(1, 5, OrderSide::Ask, 101)]);

        let policy = engine.limit_order_policy(LimitOrder::new(
            2.into(),
            Timestamp::new(0),
            5.into(),
            OrderSide::Bid,
            100.into(),
            TimeInForce::GoodTillCanceled,
        ));

        let add = ExecutionInstruction::Add(
            LimitOrder::new(
                2.into(),
                Timestamp::new(0),
                5.into(),
                OrderSide::Bid,
                100.into(),
                TimeInForce::GoodTillCanceled,
            )
            .into_book(5.into()),
        );

        assert_eq!(policy.instructions(), &[add]);
    }

    #[test]
    fn immediate_or_cancel_fills_but_never_rests() {
        let engine = engine_with(&[resting(1, 3, OrderSide::Ask, 100)]);

        let policy = engine.limit_order_policy(LimitOrder::new(
            2.into(),
            Timestamp::new(0),
            8.into(),
            OrderSide::Bid,
            100.into(),
            TimeInForce::ImmediateOrCancel,
        ));

        assert_eq!(policy.instructions(), &[fill(1, 3)]);
    }

    #[test]
    fn fill_or_kill_does_nothing_when_depth_is_insufficient() {
        let engine = engine_with(&[resting(1, 3, OrderSide::Ask, 100)]);

        let policy = engine.limit_order_policy(LimitOrder::new(
            2.into(),
            Timestamp::new(0),
            8.into(),
            OrderSide::Bid,
            100.into(),
            TimeInForce::FillOrKill,
        ));

        assert!(policy.is_empty());
    }

    #[test]
    fn fill_or_kill_matches_when_depth_is_sufficient() {
        let engine = engine_with(&[
            resting(1, 5, OrderSide::Ask, 100),
            resting(2, 5, OrderSide::Ask, 101),
        ]);

        let policy = engine.limit_order_policy(LimitOrder::new(
            9.into(),
            Timestamp::new(0),
            8.into(),
            OrderSide::Bid,
            101.into(),
            TimeInForce::FillOrKill,
        ));

        assert_eq!(policy.instructions(), &[fill(1, 5), fill(2, 3)]);
    }

    #[test]
    fn fill_or_kill_respects_the_price_bound() {
        // 8 units of depth exist, but only 3 units are reachable within the bound.
        let engine = engine_with(&[
            resting(1, 3, OrderSide::Ask, 100),
            resting(2, 5, OrderSide::Ask, 101),
        ]);

        let policy = engine.limit_order_policy(LimitOrder::new(
            9.into(),
            Timestamp::new(0),
            8.into(),
            OrderSide::Bid,
            100.into(),
            TimeInForce::FillOrKill,
        ));

        assert!(policy.is_empty());
    }

    #[test]
    fn limit_sell_crosses_the_best_bid_first() {
        let engine = engine_with(&[
            resting(1, 2, OrderSide::Bid, 100),
            resting(2, 3, OrderSide::Bid, 99),
        ]);

        let policy = engine.limit_order_policy(LimitOrder::new(
            9.into(),
            Timestamp::new(0),
            4.into(),
            OrderSide::Ask,
            99.into(),
            TimeInForce::GoodTillCanceled,
        ));

        assert_eq!(policy.instructions(), &[fill(1, 2), fill(2, 2)]);
    }

    // ── Market Orders ─────────────────────────────────────────────────────────────

    #[test]
    fn market_buy_sweeps_every_level_it_can() {
        let engine = engine_with(&[
            resting(1, 3, OrderSide::Ask, 100),
            resting(2, 3, OrderSide::Ask, 101),
            resting(3, 3, OrderSide::Ask, 102),
        ]);

        let policy = engine.market_order_policy(MarketOrder::new(
            9.into(),
            Timestamp::new(0),
            7.into(),
            OrderSide::Bid,
            MarketTimeInForce::ImmediateOrCancel,
        ));

        assert_eq!(policy.instructions(), &[fill(1, 3), fill(2, 3), fill(3, 1)]);
    }

    #[test]
    fn market_buy_never_rests_its_remainder() {
        let engine = engine_with(&[resting(1, 3, OrderSide::Ask, 100)]);

        let policy = engine.market_order_policy(MarketOrder::new(
            9.into(),
            Timestamp::new(0),
            8.into(),
            OrderSide::Bid,
            MarketTimeInForce::ImmediateOrCancel,
        ));

        assert_eq!(policy.instructions(), &[fill(1, 3)]);
    }

    #[test]
    fn market_fill_or_kill_does_nothing_on_an_empty_book() {
        let engine = engine_with(&[]);

        let policy = engine.market_order_policy(MarketOrder::new(
            9.into(),
            Timestamp::new(0),
            5.into(),
            OrderSide::Bid,
            MarketTimeInForce::FillOrKill,
        ));

        assert!(policy.is_empty());
    }

    #[test]
    fn market_fill_or_kill_matches_when_the_book_is_deep_enough() {
        let engine = engine_with(&[resting(1, 10, OrderSide::Ask, 100)]);

        let policy = engine.market_order_policy(MarketOrder::new(
            9.into(),
            Timestamp::new(0),
            5.into(),
            OrderSide::Bid,
            MarketTimeInForce::FillOrKill,
        ));

        assert_eq!(policy.instructions(), &[fill(1, 5)]);
    }

    #[test]
    fn market_order_against_an_empty_book_does_nothing() {
        let engine = engine_with(&[]);

        let policy = engine.new_order_policy(market(
            9,
            5,
            OrderSide::Bid,
            MarketTimeInForce::ImmediateOrCancel,
        ));

        assert!(policy.is_empty());
    }

    #[test]
    fn stop_orders_produce_no_instructions() {
        let engine = engine_with(&[resting(1, 5, OrderSide::Ask, 100)]);

        let stop = Order::Stop(StopOrder::new(
            9.into(),
            Timestamp::new(0),
            5.into(),
            OrderSide::Bid,
            100.into(),
            MarketTimeInForce::ImmediateOrCancel,
        ));

        assert!(engine.new_order_policy(stop).is_empty());
    }

    // ── End To End ────────────────────────────────────────────────────────────────

    #[test]
    fn submitting_a_crossing_limit_order_updates_the_book() {
        let mut engine = engine_with(&[
            resting(1, 5, OrderSide::Ask, 100),
            resting(2, 4, OrderSide::Ask, 101),
        ]);

        engine.submit_message(EngineMessage::NewOrder(limit(
            9,
            10,
            OrderSide::Bid,
            101,
            TimeInForce::GoodTillCanceled,
        )));

        // Both asks are consumed (5 + 4 = 9) and the 10th unit rests as a bid.
        assert_eq!(engine.orderbook.quantity_at(OrderSide::Ask, 100.into()), 0.into());
        assert_eq!(engine.orderbook.quantity_at(OrderSide::Ask, 101.into()), 0.into());
        assert_eq!(engine.orderbook.quantity_at(OrderSide::Bid, 101.into()), 1.into());
    }

    #[test]
    fn submitting_a_resting_limit_order_leaves_the_opposite_side_untouched() {
        let mut engine = engine_with(&[resting(1, 5, OrderSide::Ask, 100)]);

        engine.submit_message(EngineMessage::NewOrder(limit(
            9,
            5,
            OrderSide::Bid,
            99,
            TimeInForce::GoodTillCanceled,
        )));

        assert_eq!(engine.orderbook.quantity_at(OrderSide::Ask, 100.into()), 5.into());
        assert_eq!(engine.orderbook.quantity_at(OrderSide::Bid, 99.into()), 5.into());
    }

    #[test]
    fn an_order_does_not_match_against_itself() {
        // A bid resting at 100 must not cross the ask it creates in the same policy.
        let mut engine = engine_with(&[]);

        engine.submit_message(EngineMessage::NewOrder(limit(
            9,
            5,
            OrderSide::Bid,
            100,
            TimeInForce::GoodTillCanceled,
        )));

        assert_eq!(engine.orderbook.quantity_at(OrderSide::Bid, 100.into()), 5.into());
        assert_eq!(engine.orderbook.quantity_at(OrderSide::Ask, 100.into()), 0.into());
    }
}
