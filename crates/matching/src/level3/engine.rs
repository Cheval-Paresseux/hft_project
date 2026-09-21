use std::collections::HashMap;

use orderbook::{
    level3::L3OrderBook,
    order::{OrderId, OrderSide, Price, Quantity},
};

use crate::order::{
    LimitOrder, MarketOrder, MarketTimeInForce, Order, StopLimitOrder, StopOrder, TimeInForce,
};

// ── Matching Engine ───────────────────────────────────────────────────────────

pub struct L3MatchingEngine<OB: L3OrderBook> {
    orderbook: OB,
    stop_book: StopBook,
}

impl<OB: L3OrderBook> Default for L3MatchingEngine<OB> {
    fn default() -> Self {
        Self {
            orderbook: OB::default(),
            stop_book: StopBook::default(),
        }
    }
}

// ── Stop Book ─────────────────────────────────────────────────────────────────

#[derive(Default)]
struct StopBook {
    stops: HashMap<OrderId, Order>,
}

// ── Order Execution ───────────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    pub fn submit_order(&mut self, order: Order) {
        match order {
            Order::Market(order) => self.execute_market(order),
            Order::Limit(order) => self.execute_limit(order),
            Order::Stop(order) => self.execute_stop(order),
            Order::StopLimit(order) => self.execute_stop_limit(order),
        }

        self.trigger_stops();
    }

    pub fn cancel_order(&mut self, order_id: OrderId) {
        let _ = self.orderbook.cancel_order(order_id);
    }

    pub fn modify_order(&mut self, order_id: OrderId, new_quantity: Quantity) {
        let _ = self.orderbook.modify_order(order_id, new_quantity);
    }
}

// ── Execution Paths ───────────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    fn execute_market(&mut self, order: MarketOrder) {
        if !self.pre_check(
            order.side,
            order.quantity,
            None,
            order.tif == MarketTimeInForce::FillOrKill,
        ) {
            return;
        }

        self.fill(order.side, order.quantity, None);
    }

    fn execute_limit(&mut self, order: LimitOrder) {
        if !self.pre_check(
            order.side,
            order.quantity,
            Some(order.price),
            order.tif == TimeInForce::FillOrKill,
        ) {
            return;
        }

        let residual = self.fill(order.side, order.quantity, Some(order.price));

        self.post_check(order, residual);
    }

    fn execute_stop(&mut self, order: StopOrder) {
        if self.stop_crossed(order.side, order.trigger) {
            self.execute_market(order.into());
        } else {
            self.stop_book.stops.insert(order.id, Order::Stop(order));
        }
    }

    fn execute_stop_limit(&mut self, order: StopLimitOrder) {
        if self.stop_crossed(order.side, order.trigger) {
            self.execute_limit(order.into());
        } else {
            self.stop_book
                .stops
                .insert(order.id, Order::StopLimit(order));
        }
    }
}

// ── Execution Phases ──────────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    fn pre_check(
        &self,
        side: OrderSide,
        quantity: Quantity,
        limit: Option<Price>,
        fok: bool,
    ) -> bool {
        !fok || self.orderbook.available_quantity(side.opposite(), limit) >= quantity
    }

    fn fill(&mut self, side: OrderSide, mut incoming: Quantity, limit: Option<Price>) -> Quantity {
        while !incoming.is_zero() {
            let Some((rest_id, rest_price, rest_quantity)) = self.orderbook.best(side.opposite())
            else {
                break;
            };

            if limit.is_some_and(|limit| !crosses(side, limit, rest_price)) {
                break;
            }

            let fill_quantity = Quantity::new(incoming.get().min(rest_quantity.get()));

            self.orderbook
                .fill(rest_id, fill_quantity)
                .expect("resting order must still be in the book");

            incoming -= fill_quantity;

            if fill_quantity == rest_quantity {
                self.orderbook
                    .cancel_order(rest_id)
                    .expect("fully-filled resting order must be removed");
            }
        }

        incoming
    }

    fn post_check(&mut self, order: LimitOrder, residual: Quantity) {
        if residual.is_zero() {
            return;
        }

        match order.tif {
            TimeInForce::Day | TimeInForce::GoodTillCanceled => {
                self.orderbook.add_order(order.into_book(residual));
            }
            TimeInForce::FillOrKill | TimeInForce::ImmediateOrCancel => {}
        }
    }
}

// ── Stop Activation ───────────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    fn trigger_stops(&mut self) {
        let triggered: Vec<Order> = self
            .stop_book
            .stops
            .values()
            .copied()
            .filter(|order| match order {
                Order::Stop(order) => self.stop_crossed(order.side, order.trigger),
                Order::StopLimit(order) => self.stop_crossed(order.side, order.trigger),
                _ => false,
            })
            .collect();

        for order in triggered {
            self.stop_book.stops.remove(&order.id());

            match order {
                Order::Stop(order) => self.execute_market(order.into()),
                Order::StopLimit(order) => self.execute_limit(order.into()),
                _ => unreachable!("the stop book only holds stop orders"),
            }
        }
    }

    fn stop_crossed(&self, side: OrderSide, trigger: Price) -> bool {
        match side {
            OrderSide::Bid => self
                .orderbook
                .best_price(OrderSide::Ask)
                .is_some_and(|ask| ask >= trigger),
            OrderSide::Ask => self
                .orderbook
                .best_price(OrderSide::Bid)
                .is_some_and(|bid| bid <= trigger),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn crosses(side: OrderSide, limit: Price, rest_price: Price) -> bool {
    match side {
        OrderSide::Bid => rest_price <= limit,
        OrderSide::Ask => rest_price >= limit,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use orderbook::level3::vec::VecL3OrderBook;
    use orderbook::order::Timestamp;

    fn seed(
        engine: &mut L3MatchingEngine<VecL3OrderBook>,
        id: u64,
        quantity: u64,
        side: OrderSide,
        price: u64,
    ) {
        let order = Order::Limit(LimitOrder::new(
            OrderId::new(id),
            Timestamp::new(0),
            Quantity::new(quantity),
            side,
            Price::new(price),
            TimeInForce::Day,
        ));
        engine.submit_order(order);
    }

    fn limit_buy(id: u64, quantity: u64, price: u64, tif: TimeInForce) -> Order {
        Order::Limit(LimitOrder::new(
            OrderId::new(id),
            Timestamp::new(1),
            Quantity::new(quantity),
            OrderSide::Bid,
            Price::new(price),
            tif,
        ))
    }

    fn market_buy(id: u64, quantity: u64, tif: MarketTimeInForce) -> Order {
        Order::Market(MarketOrder::new(
            OrderId::new(id),
            Timestamp::new(1),
            Quantity::new(quantity),
            OrderSide::Bid,
            tif,
        ))
    }

    #[test]
    fn market_sweeps_and_drops_residual() {
        let mut engine = L3MatchingEngine::default();
        seed(&mut engine, 1, 5, OrderSide::Ask, 100);
        seed(&mut engine, 2, 10, OrderSide::Ask, 101);

        engine.submit_order(market_buy(3, 20, MarketTimeInForce::ImmediateOrCancel));

        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 100.into())
                .get(),
            0
        );
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 101.into())
                .get(),
            0
        );
    }

    #[test]
    fn market_fok_kills_when_insufficient_liquidity() {
        let mut engine = L3MatchingEngine::default();
        seed(&mut engine, 1, 5, OrderSide::Ask, 100);
        seed(&mut engine, 2, 5, OrderSide::Ask, 101);

        engine.submit_order(market_buy(3, 11, MarketTimeInForce::FillOrKill));

        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 100.into())
                .get(),
            5
        );
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 101.into())
                .get(),
            5
        );
    }

    #[test]
    fn market_fok_fills_when_sufficient_liquidity() {
        let mut engine = L3MatchingEngine::default();
        seed(&mut engine, 1, 5, OrderSide::Ask, 100);
        seed(&mut engine, 2, 5, OrderSide::Ask, 101);

        engine.submit_order(market_buy(3, 10, MarketTimeInForce::FillOrKill));

        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 100.into())
                .get(),
            0
        );
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 101.into())
                .get(),
            0
        );
    }

    #[test]
    fn limit_ioc_drops_residual() {
        let mut engine = L3MatchingEngine::default();
        seed(&mut engine, 1, 5, OrderSide::Ask, 100);
        seed(&mut engine, 2, 10, OrderSide::Ask, 101);

        engine.submit_order(limit_buy(3, 8, 100, TimeInForce::ImmediateOrCancel));

        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 100.into())
                .get(),
            0
        );
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 101.into())
                .get(),
            10
        );
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Bid, 100.into())
                .get(),
            0
        );
    }

    #[test]
    fn limit_day_rests_residual() {
        let mut engine = L3MatchingEngine::default();
        seed(&mut engine, 1, 5, OrderSide::Ask, 100);

        engine.submit_order(limit_buy(2, 8, 100, TimeInForce::Day));

        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 100.into())
                .get(),
            0
        );
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Bid, 100.into())
                .get(),
            3
        );
    }

    #[test]
    fn limit_fok_respects_price_bound() {
        let mut engine = L3MatchingEngine::default();
        seed(&mut engine, 1, 5, OrderSide::Ask, 100);
        seed(&mut engine, 2, 5, OrderSide::Ask, 101);

        // 10 needed but only 5 sits at or below the limit: killed, book untouched
        engine.submit_order(limit_buy(3, 10, 100, TimeInForce::FillOrKill));
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 100.into())
                .get(),
            5
        );
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 101.into())
                .get(),
            5
        );

        // same size with a wide enough limit: fills fully, nothing rests
        engine.submit_order(limit_buy(4, 10, 101, TimeInForce::FillOrKill));
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 100.into())
                .get(),
            0
        );
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 101.into())
                .get(),
            0
        );
    }

    #[test]
    fn stop_triggers_immediately_when_crossed() {
        let mut engine = L3MatchingEngine::default();
        seed(&mut engine, 1, 5, OrderSide::Ask, 100);

        // buy stop at 100: best ask is already 100, so it fires right away
        let stop = Order::Stop(StopOrder::new(
            OrderId::new(2),
            Timestamp::new(1),
            Quantity::new(3),
            OrderSide::Bid,
            Price::new(100),
            MarketTimeInForce::ImmediateOrCancel,
        ));
        engine.submit_order(stop);

        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 100.into())
                .get(),
            2
        );
    }

    #[test]
    fn stop_parks_then_triggers() {
        let mut engine = L3MatchingEngine::default();
        seed(&mut engine, 1, 5, OrderSide::Bid, 99);

        // sell stop at 95: best bid (99) is above the trigger, so it parks
        let stop = Order::Stop(StopOrder::new(
            OrderId::new(2),
            Timestamp::new(1),
            Quantity::new(5),
            OrderSide::Ask,
            Price::new(95),
            MarketTimeInForce::ImmediateOrCancel,
        ));
        engine.submit_order(stop);
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Bid, 99.into())
                .get(),
            5
        );

        // a market sell consumes the bid liquidity above the trigger
        let sell = Order::Market(MarketOrder::new(
            OrderId::new(3),
            Timestamp::new(1),
            Quantity::new(5),
            OrderSide::Ask,
            MarketTimeInForce::ImmediateOrCancel,
        ));
        engine.submit_order(sell);
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Bid, 99.into())
                .get(),
            0
        );

        // a fresh bid below the trigger activates the parked sell stop,
        // which then fills 5 of the 10 printed
        seed(&mut engine, 4, 10, OrderSide::Bid, 94);
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Bid, 94.into())
                .get(),
            5
        );
    }

    #[test]
    fn stop_limit_triggers_as_limit() {
        let mut engine = L3MatchingEngine::default();
        seed(&mut engine, 1, 5, OrderSide::Ask, 100);

        // buy stop-limit: trigger 100, limit 100, already crossed -> fills as a limit
        let stop = Order::StopLimit(StopLimitOrder::new(
            OrderId::new(2),
            Timestamp::new(1),
            Quantity::new(3),
            OrderSide::Bid,
            Price::new(100),
            Price::new(100),
            TimeInForce::Day,
        ));
        engine.submit_order(stop);

        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 100.into())
                .get(),
            2
        );
    }

    #[test]
    fn stop_limit_parks_when_not_crossed() {
        let mut engine = L3MatchingEngine::default();
        seed(&mut engine, 1, 5, OrderSide::Ask, 100);

        // buy stop-limit at 101: best ask (100) is below the trigger, so it parks
        let stop = Order::StopLimit(StopLimitOrder::new(
            OrderId::new(2),
            Timestamp::new(1),
            Quantity::new(3),
            OrderSide::Bid,
            Price::new(101),
            Price::new(101),
            TimeInForce::Day,
        ));
        engine.submit_order(stop);

        // parked: nothing was filled, and no bid was printed
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Ask, 100.into())
                .get(),
            5
        );
        assert_eq!(
            engine
                .orderbook
                .quantity_at(OrderSide::Bid, 101.into())
                .get(),
            0
        );
    }
}
