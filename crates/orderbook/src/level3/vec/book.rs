use super::side::VecL3BookSide;

use crate::{
    errors::OrderBookError,
    level3::traits::L3OrderBook,
    order::{LimitOrder, OrderId, OrderSide, Price, Quantity},
};

use std::collections::HashMap;

// ── Order Book ────────────────────────────────────────────────────────────────

pub struct VecL3OrderBook {
    ask_side: VecL3BookSide,
    bid_side: VecL3BookSide,
    orders_map: HashMap<OrderId, (OrderSide, Price)>,
}

impl Default for VecL3OrderBook {
    fn default() -> Self {
        Self {
            ask_side: VecL3BookSide::new(OrderSide::Ask),
            bid_side: VecL3BookSide::new(OrderSide::Bid),
            orders_map: HashMap::new(),
        }
    }
}

impl VecL3OrderBook {
    fn find_order(&self, order_id: OrderId) -> Result<&(OrderSide, Price), OrderBookError> {
        self.orders_map
            .get(&order_id)
            .ok_or(OrderBookError::OrderIdNotFound { order_id })
    }
}

impl L3OrderBook for VecL3OrderBook {
    fn add_order(&mut self, order: LimitOrder) {
        self.orders_map.insert(order.id, (order.side, order.price));

        match order.side {
            OrderSide::Bid => self.bid_side.add_order(order),
            OrderSide::Ask => self.ask_side.add_order(order),
        }
    }

    fn cancel_order(&mut self, order_id: OrderId) -> Result<(), OrderBookError> {
        let &(side, price) = self.find_order(order_id)?;

        match side {
            OrderSide::Bid => self.bid_side.cancel_order(order_id, price)?,
            OrderSide::Ask => self.ask_side.cancel_order(order_id, price)?,
        }

        self.orders_map.remove(&order_id);

        Ok(())
    }

    fn modify_order(
        &mut self,
        order_id: OrderId,
        new_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let &(side, price) = self.find_order(order_id)?;

        match side {
            OrderSide::Bid => self.bid_side.modify_order(order_id, price, new_quantity),
            OrderSide::Ask => self.ask_side.modify_order(order_id, price, new_quantity),
        }
    }

    fn fill(&mut self, _order_id: OrderId, _quantity: Quantity) -> Result<(), OrderBookError> {
        todo!()
    }

    fn best(&self, _side: OrderSide) -> Option<(OrderId, Price, Quantity)> {
        todo!()
    }

    fn best_price(&self, _side: OrderSide) -> Option<Price> {
        todo!()
    }

    fn quantity_at(&self, _side: OrderSide, _price: Price) -> Quantity {
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::OrderSide;

    fn make_limit(
        id: u64,
        timestamp: u64,
        quantity: u64,
        side: OrderSide,
        price: u64,
    ) -> LimitOrder {
        LimitOrder::new(
            id.into(),
            timestamp.into(),
            quantity.into(),
            side,
            price.into(),
        )
    }

    fn make_book() -> VecL3OrderBook {
        VecL3OrderBook::default()
    }

    #[test]
    fn add_order() {
        let mut book = make_book();
        let limit_order = make_limit(0, 0, 10, OrderSide::Bid, 100);

        book.add_order(limit_order);

        assert!(!book.bid_side.is_empty());
        assert!(book.find_order(0.into()).is_ok());
    }

    #[test]
    fn cancel_order() {
        let mut book = make_book();
        let limit_order = make_limit(0, 0, 10, OrderSide::Bid, 100);

        book.add_order(limit_order);
        let result = book.cancel_order(0.into());

        assert!(result.is_ok());
        assert!(book.bid_side.is_empty());
        assert!(book.find_order(0.into()).is_err());
    }

    #[test]
    fn modify_order() {
        let mut book = make_book();
        let limit_order = make_limit(0, 0, 10, OrderSide::Bid, 100);

        book.add_order(limit_order);
        let result = book.modify_order(0.into(), 23.into());

        assert!(result.is_ok());
        assert!(!book.bid_side.is_empty());
        assert!(book.find_order(0.into()).is_ok());
    }
}
