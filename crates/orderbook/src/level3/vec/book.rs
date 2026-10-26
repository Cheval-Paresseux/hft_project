use super::side::VecL3BookSide;

use crate::level3::common::{L3OrderBook, OrderBookError};

use types::order::{
    fields::{OrderId, OrderSide, Price, Quantity},
    orders::{LimitOrder, Order},
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
    // ── Book Mutations ────────────────────────────────────────────────────────────

    fn add_order(&mut self, order: LimitOrder) -> Result<(), OrderBookError> {
        if self.orders_map.contains_key(&order.id()) {
            return Err(OrderBookError::OrderIdAlreadyExists {
                order_id: order.id(),
            });
        }

        match order.side() {
            OrderSide::Bid => self.bid_side.add_order(order)?,
            OrderSide::Ask => self.ask_side.add_order(order)?,
        }

        self.orders_map
            .insert(order.id(), (order.side(), order.limit()));

        Ok(())
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

    fn fill_order(
        &mut self,
        order_id: OrderId,
        fill_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let &(side, price) = self.find_order(order_id)?;

        match side {
            OrderSide::Bid => self.bid_side.fill_order(order_id, price, fill_quantity)?,
            OrderSide::Ask => self.ask_side.fill_order(order_id, price, fill_quantity)?,
        }

        if self.order(order_id).is_none() {
            self.orders_map.remove(&order_id);
        }

        Ok(())
    }

    // ── Orders Look Up  ───────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(OrderSide, Price, Quantity)> {
        let &(side, price) = self.find_order(order_id).ok()?;

        match side {
            OrderSide::Bid => self.bid_side.order(order_id, price),
            OrderSide::Ask => self.ask_side.order(order_id, price),
        }
    }

    fn top_order(&self, side: OrderSide) -> Option<(OrderId, Price, Quantity)> {
        match side {
            OrderSide::Bid => self.bid_side.top_order(),
            OrderSide::Ask => self.ask_side.top_order(),
        }
    }

    fn orders_at(&self, side: OrderSide, price: Price) -> Option<Vec<(OrderId, Quantity)>> {
        match side {
            OrderSide::Bid => self.bid_side.orders_at(price),
            OrderSide::Ask => self.ask_side.orders_at(price),
        }
    }

    fn orders_up_to(
        &self,
        side: OrderSide,
        bound: Option<Price>,
    ) -> Option<Vec<(OrderId, Price, Quantity)>> {
        match side {
            OrderSide::Bid => self.bid_side.orders_up_to(bound),
            OrderSide::Ask => self.ask_side.orders_up_to(bound),
        }
    }

    fn orders_up_to_quantity(
        &self,
        side: OrderSide,
        bound: Option<Quantity>,
    ) -> Option<Vec<(OrderId, Price, Quantity)>> {
        match side {
            OrderSide::Bid => self.bid_side.orders_up_to_quantity(bound),
            OrderSide::Ask => self.ask_side.orders_up_to_quantity(bound),
        }
    }

    // ── Aggregation Look Up ───────────────────────────────────────────────────────

    fn top_price(&self, side: OrderSide) -> Option<Price> {
        match side {
            OrderSide::Bid => self.bid_side.top_price(),
            OrderSide::Ask => self.ask_side.top_price(),
        }
    }

    fn quantity_at(&self, side: OrderSide, price: Price) -> Option<Quantity> {
        match side {
            OrderSide::Bid => self.bid_side.quantity_at(price),
            OrderSide::Ask => self.ask_side.quantity_at(price),
        }
    }

    fn quantity_up_to(&self, side: OrderSide, bound: Option<Price>) -> Option<Quantity> {
        match side {
            OrderSide::Bid => self.bid_side.quantity_up_to(bound),
            OrderSide::Ask => self.ask_side.quantity_up_to(bound),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use types::order::TimeInForce;

    use super::*;

    fn make_limit(id: u64, quantity: u64, side: OrderSide, price: u64) -> LimitOrder {
        LimitOrder::new(
            id.into(),
            quantity.into(),
            0.into(),
            side,
            price.into(),
            TimeInForce::GoodTillCanceled,
        )
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

        assert_eq!(
            book.fill_order(42.into(), 5.into()),
            Err(OrderBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
    }

    #[test]
    fn add_order() {
        let mut book = VecL3OrderBook::default();

        assert_eq!(
            book.add_order(make_limit(1, 10, OrderSide::Bid, 100)),
            Ok(())
        );
        assert_eq!(
            book.add_order(make_limit(2, 10, OrderSide::Ask, 200)),
            Ok(())
        );

        assert_eq!(book.orders_map[&1.into()], (OrderSide::Bid, 100.into()));
        assert_eq!(book.orders_map[&2.into()], (OrderSide::Ask, 200.into()));
    }

    #[test]
    fn add_duplicate_order_id_errors() {
        let mut book = VecL3OrderBook::default();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(
            book.add_order(make_limit(1, 5, OrderSide::Bid, 105)),
            Err(OrderBookError::OrderIdAlreadyExists { order_id: 1.into() })
        );

        assert_eq!(
            book.order(1.into()),
            Some((OrderSide::Bid, 100.into(), 10.into()))
        );

        assert_eq!(book.top_price(OrderSide::Bid), Some(100.into()));
        assert_eq!(book.quantity_at(OrderSide::Bid, 105.into()), None);
    }

    #[test]
    fn failed_add_order_does_not_update_index() {
        let mut book = VecL3OrderBook::default();
        book.add_order(make_limit(1, u64::MAX, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(
            book.add_order(make_limit(2, 1, OrderSide::Bid, 100)),
            Err(OrderBookError::QuantityOverflow)
        );

        assert_eq!(book.order(2.into()), None);
        assert_eq!(
            book.quantity_at(OrderSide::Bid, 100.into()),
            Some(u64::MAX.into())
        );
    }

    #[test]
    fn cancel_order() {
        let mut book = VecL3OrderBook::default();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(book.cancel_order(1.into()), Ok(()));
        assert!(book.orders_map.is_empty());
    }

    #[test]
    fn modify_order() {
        let mut book = VecL3OrderBook::default();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(book.modify_order(1.into(), 23.into()), Ok(()));
        assert_eq!(
            book.order(1.into()),
            Some((OrderSide::Bid, 100.into(), 23.into()))
        );
    }

    #[test]
    fn fill_order() {
        let mut book = VecL3OrderBook::default();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(book.fill_order(1.into(), 4.into()), Ok(()));
        assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), Some(6.into()));
    }

    #[test]
    fn fully_filled_order_is_removed() {
        let mut book = VecL3OrderBook::default();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(book.fill_order(1.into(), 10.into()), Ok(()));

        assert_eq!(book.order(1.into()), None);
        assert_eq!(book.top_order(OrderSide::Bid), None);
        assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), None);
        assert!(!book.orders_map.contains_key(&1.into()));
    }

    #[test]
    fn order() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        let expected_result = (OrderSide::Bid, Price::new(100), Quantity::new(10));
        assert_eq!(book.order(1.into()), Some(expected_result));
        assert_eq!(book.order(42.into()), None);
    }

    #[test]
    fn top_order() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();
        book.add_order(make_limit(2, 10, OrderSide::Bid, 105))
            .unwrap();
        book.add_order(make_limit(3, 10, OrderSide::Ask, 200))
            .unwrap();
        book.add_order(make_limit(4, 10, OrderSide::Ask, 195))
            .unwrap();

        assert_eq!(
            book.top_order(OrderSide::Bid),
            Some((2.into(), 105.into(), 10.into()))
        );
        assert_eq!(
            book.top_order(OrderSide::Ask),
            Some((4.into(), 195.into(), 10.into()))
        );
    }

    #[test]
    fn orders_at() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();
        book.add_order(make_limit(2, 10, OrderSide::Bid, 100))
            .unwrap();

        let expected_result = vec![(1.into(), 10.into()), (2.into(), 10.into())];
        assert_eq!(
            book.orders_at(OrderSide::Bid, 100.into()),
            Some(expected_result)
        );
        assert_eq!(book.orders_at(OrderSide::Bid, 42.into()), None);
    }

    #[test]
    fn orders_up_to() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();
        book.add_order(make_limit(2, 5, OrderSide::Bid, 105))
            .unwrap();
        book.add_order(make_limit(3, 7, OrderSide::Ask, 200))
            .unwrap();
        book.add_order(make_limit(4, 8, OrderSide::Ask, 195))
            .unwrap();

        assert_eq!(
            book.orders_up_to(OrderSide::Bid, None),
            Some(vec![
                (2.into(), 105.into(), 5.into()),
                (1.into(), 100.into(), 10.into()),
            ])
        );
        assert_eq!(
            book.orders_up_to(OrderSide::Ask, None),
            Some(vec![
                (4.into(), 195.into(), 8.into()),
                (3.into(), 200.into(), 7.into()),
            ])
        );
        assert_eq!(
            book.orders_up_to(OrderSide::Ask, Some(195.into())),
            Some(vec![(4.into(), 195.into(), 8.into())])
        );
        assert_eq!(book.orders_up_to(OrderSide::Bid, Some(106.into())), None);
    }

    #[test]
    fn orders_up_to_quantity() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();
        book.add_order(make_limit(2, 5, OrderSide::Bid, 105))
            .unwrap();
        book.add_order(make_limit(3, 7, OrderSide::Ask, 200))
            .unwrap();
        book.add_order(make_limit(4, 8, OrderSide::Ask, 195))
            .unwrap();

        assert_eq!(
            book.orders_up_to_quantity(OrderSide::Bid, None),
            Some(vec![
                (2.into(), 105.into(), 5.into()),
                (1.into(), 100.into(), 10.into()),
            ])
        );
        assert_eq!(
            book.orders_up_to_quantity(OrderSide::Ask, None),
            Some(vec![
                (4.into(), 195.into(), 8.into()),
                (3.into(), 200.into(), 7.into()),
            ])
        );
        assert_eq!(
            book.orders_up_to_quantity(OrderSide::Ask, Some(8.into())),
            Some(vec![(4.into(), 195.into(), 8.into())])
        );
        assert_eq!(
            book.orders_up_to_quantity(OrderSide::Ask, Some(12.into())),
            Some(vec![
                (4.into(), 195.into(), 8.into()),
                (3.into(), 200.into(), 7.into()),
            ])
        );
        assert_eq!(
            book.orders_up_to_quantity(OrderSide::Bid, Some(5.into())),
            Some(vec![(2.into(), 105.into(), 5.into())])
        );
        assert_eq!(
            book.orders_up_to_quantity(OrderSide::Bid, Some(100.into())),
            book.orders_up_to_quantity(OrderSide::Bid, None)
        );
        assert_eq!(
            VecL3OrderBook::default().orders_up_to_quantity(OrderSide::Bid, Some(1.into())),
            None
        );
    }

    #[test]
    fn top_price() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();
        book.add_order(make_limit(2, 10, OrderSide::Bid, 105))
            .unwrap();
        book.add_order(make_limit(3, 10, OrderSide::Ask, 200))
            .unwrap();
        book.add_order(make_limit(4, 10, OrderSide::Ask, 195))
            .unwrap();

        assert_eq!(book.top_price(OrderSide::Bid), Some(105.into()));
        assert_eq!(book.top_price(OrderSide::Ask), Some(195.into()));

        assert_eq!(VecL3OrderBook::default().top_price(OrderSide::Bid), None);
    }

    #[test]
    fn quantity_at() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();
        book.add_order(make_limit(2, 5, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(
            book.quantity_at(OrderSide::Bid, 100.into()),
            Some(15.into())
        );
        assert_eq!(book.quantity_at(OrderSide::Bid, 42.into()), None);
        assert_eq!(book.quantity_at(OrderSide::Ask, 100.into()), None);
    }

    #[test]
    fn quantity_up_to() {
        let mut book = VecL3OrderBook::default();

        book.add_order(make_limit(1, 10, OrderSide::Ask, 100))
            .unwrap();
        book.add_order(make_limit(2, 5, OrderSide::Ask, 101))
            .unwrap();
        book.add_order(make_limit(3, 7, OrderSide::Ask, 103))
            .unwrap();
        book.add_order(make_limit(4, 8, OrderSide::Bid, 100))
            .unwrap();
        book.add_order(make_limit(5, 8, OrderSide::Bid, 99))
            .unwrap();

        assert_eq!(book.quantity_up_to(OrderSide::Ask, None), Some(22.into()));
        assert_eq!(
            book.quantity_up_to(OrderSide::Ask, Some(101.into())),
            Some(15.into())
        );
        assert_eq!(
            book.quantity_up_to(OrderSide::Ask, Some(100.into())),
            Some(10.into())
        );
        assert_eq!(
            book.quantity_up_to(OrderSide::Ask, Some(99.into())),
            Some(0.into())
        );

        assert_eq!(book.quantity_up_to(OrderSide::Bid, None), Some(16.into()));
        assert_eq!(
            book.quantity_up_to(OrderSide::Bid, Some(100.into())),
            Some(8.into())
        );
        assert_eq!(
            book.quantity_up_to(OrderSide::Bid, Some(99.into())),
            Some(16.into())
        );
        assert_eq!(
            book.quantity_up_to(OrderSide::Bid, Some(101.into())),
            Some(0.into())
        );
    }
}
