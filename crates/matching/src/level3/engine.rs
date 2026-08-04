use orderbook::{
    level3::L3OrderBook,
    order::{OrderId, Quantity},
};

use crate::order::{Order, OrderKind};

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

// ── Order Execution ───────────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    pub fn submit_order(&mut self, order: Order) {
        match order.kind {
            OrderKind::Market => self.execute_market(order),
            OrderKind::Limit => self.execute_limit(order),
            OrderKind::Stop(_) => self.execute_stop(order),
            OrderKind::StopLimit(_) => self.execute_stop_limit(order),
        }
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
    fn execute_market(&mut self, mut order: Order) {
        while !order.quantity.is_zero() {
            let Some((rest_id, _rest_price, rest_quantity)) =
                self.orderbook.best(order.side.opposite())
            else {
                break;
            };

            let fill_quantity = Quantity::new(order.quantity.get().min(rest_quantity.get()));

            self.orderbook
                .fill(rest_id, fill_quantity)
                .expect("resting order must still be in the book");

            order.quantity -= fill_quantity;

            if fill_quantity == rest_quantity {
                self.orderbook
                    .cancel_order(rest_id)
                    .expect("fully-filled resting order must be removed");
            }
        }
    }

    fn execute_limit(&mut self, _order: Order) {
        todo!()
    }

    fn execute_stop(&mut self, _order: Order) {
        todo!()
    }

    fn execute_stop_limit(&mut self, _order: Order) {
        todo!()
    }
}
