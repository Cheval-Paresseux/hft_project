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

    fn fill(&mut self, order_id: OrderId, fill_quantity: Quantity) -> Result<(), OrderBookError> {
        let &(side, price) = self.find_order(order_id)?;

        match side {
            OrderSide::Bid => self.bid_side.fill_order(order_id, price, fill_quantity),
            OrderSide::Ask => self.ask_side.fill_order(order_id, price, fill_quantity),
        }
    }

    fn best(&self, side: OrderSide) -> Option<(OrderId, Price, Quantity)> {
        match side {
            OrderSide::Bid => self.bid_side.best(),
            OrderSide::Ask => self.ask_side.best(),
        }
    }

    fn best_price(&self, side: OrderSide) -> Option<Price> {
        match side {
            OrderSide::Bid => self.bid_side.best_price(),
            OrderSide::Ask => self.ask_side.best_price(),
        }
    }

    fn quantity_at(&self, side: OrderSide, price: Price) -> Quantity {
        match side {
            OrderSide::Bid => self.bid_side.quantity_at(price),
            OrderSide::Ask => self.ask_side.quantity_at(price),
        }
    }

    fn available_quantity(&self, side: OrderSide, bound: Option<Price>) -> Quantity {
        match side {
            OrderSide::Bid => self.bid_side.available_quantity(bound),
            OrderSide::Ask => self.ask_side.available_quantity(bound),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_limit(id: u64, quantity: u64, side: OrderSide, price: u64) -> LimitOrder {
        LimitOrder::new(id.into(), 0.into(), quantity.into(), side, price.into())
    }

    #[test]
    fn add_order() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100));
        book.add_order(make_limit(2, 10, OrderSide::Ask, 200));

        assert_eq!(book.orders_map[&1.into()], (OrderSide::Bid, 100.into()));
        assert_eq!(book.orders_map[&2.into()], (OrderSide::Ask, 200.into()));
    }

    #[test]
    fn cancel_order() {
        let mut book = VecL3OrderBook::default();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100));

        assert_eq!(book.cancel_order(1.into()), Ok(()));
        assert!(book.orders_map.is_empty());
    }

    #[test]
    fn modify_order() {
        let mut book = VecL3OrderBook::default();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100));

        assert_eq!(book.modify_order(1.into(), 23.into()), Ok(()));
        assert!(book.find_order(1.into()).is_ok());
    }

    #[test]
    fn best_and_best_price() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100));
        book.add_order(make_limit(2, 10, OrderSide::Bid, 105));
        book.add_order(make_limit(3, 10, OrderSide::Ask, 200));
        book.add_order(make_limit(4, 10, OrderSide::Ask, 195));

        assert_eq!(book.best(OrderSide::Bid), Some((2.into(), 105.into(), 10.into())));
        assert_eq!(book.best_price(OrderSide::Bid), Some(105.into()));
        assert_eq!(book.best(OrderSide::Ask), Some((4.into(), 195.into(), 10.into())));
        assert_eq!(book.best_price(OrderSide::Ask), Some(195.into()));
    }

    #[test]
    fn quantity_at() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100));
        book.add_order(make_limit(2, 5, OrderSide::Bid, 100));

        assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), 15.into());
        assert_eq!(book.quantity_at(OrderSide::Ask, 100.into()), 0.into());
    }

    #[test]
    fn available_quantity() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Ask, 100));
        book.add_order(make_limit(2, 5, OrderSide::Ask, 101));
        book.add_order(make_limit(3, 7, OrderSide::Ask, 103));
        book.add_order(make_limit(4, 8, OrderSide::Bid, 100));
        book.add_order(make_limit(5, 8, OrderSide::Bid, 99));

        assert_eq!(book.available_quantity(OrderSide::Ask, None), 22.into());
        assert_eq!(book.available_quantity(OrderSide::Ask, Some(101.into())), 15.into());
        assert_eq!(book.available_quantity(OrderSide::Ask, Some(100.into())), 10.into());
        assert_eq!(book.available_quantity(OrderSide::Ask, Some(99.into())), 0.into());

        assert_eq!(book.available_quantity(OrderSide::Bid, None), 16.into());
        assert_eq!(book.available_quantity(OrderSide::Bid, Some(100.into())), 8.into());
        assert_eq!(book.available_quantity(OrderSide::Bid, Some(99.into())), 16.into());
    }

    #[test]
    fn fill() {
        let mut book = VecL3OrderBook::default();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100));

        assert_eq!(book.fill(1.into(), 4.into()), Ok(()));
        assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), 6.into());

        assert_eq!(
            book.fill(42.into(), 1.into()),
            Err(OrderBookError::OrderIdNotFound { order_id: 42.into() })
        );
    }

    #[test]
    fn unknown_order_errors() {
        let mut book = VecL3OrderBook::default();

        assert_eq!(
            book.cancel_order(42.into()),
            Err(OrderBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
        assert_eq!(
            book.modify_order(42.into(), 5.into()),
            Err(OrderBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
    }
}
