use super::side::VecL3BookSide;

use crate::level3::common::{L3OrderBook, OrderBookError};

use types::order::{
    fields::{OrderId, OrderSide, Price, Quantity},
    orders::{IncomingOrder, Order},
};

use std::{collections::HashMap, marker::PhantomData};

// ── Order Book ────────────────────────────────────────────────────────────────

pub struct VecL3OrderBook<I, R> {
    ask_side: VecL3BookSide<R>,
    bid_side: VecL3BookSide<R>,
    orders_map: HashMap<OrderId, (OrderSide, Price)>,

    _incoming: PhantomData<I>,
}

impl<I, R> Default for VecL3OrderBook<I, R> {
    fn default() -> Self {
        Self {
            ask_side: VecL3BookSide::new(OrderSide::Ask),
            bid_side: VecL3BookSide::new(OrderSide::Bid),
            orders_map: HashMap::new(),

            _incoming: PhantomData,
        }
    }
}

impl<I, R> VecL3OrderBook<I, R> {
    fn find_order(&self, order_id: OrderId) -> Result<&(OrderSide, Price), OrderBookError> {
        self.orders_map
            .get(&order_id)
            .ok_or(OrderBookError::OrderIdNotFound { order_id })
    }
}

impl<I, R> L3OrderBook for VecL3OrderBook<I, R>
where
    I: IncomingOrder,
    R: Order + From<I>,
{
    type Incoming = I;
    type Resting = R;

    // ── Book Mutations ────────────────────────────────────────────────────────

    fn add_order(&mut self, order: I) -> Result<(), OrderBookError> {
        let order_id = order.id();
        let side = order.side();
        let price = order.limit();

        if self.orders_map.contains_key(&order_id) {
            return Err(OrderBookError::OrderIdAlreadyExists { order_id });
        }

        match side {
            OrderSide::Bid => self.bid_side.add_order(order)?,
            OrderSide::Ask => self.ask_side.add_order(order)?,
        }

        self.orders_map.insert(order_id, (side, price));

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

    // ── Get Order ─────────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(OrderSide, Price, &R)> {
        let &(side, price) = self.find_order(order_id).ok()?;

        match side {
            OrderSide::Bid => self.bid_side.order(order_id, price),
            OrderSide::Ask => self.ask_side.order(order_id, price),
        }
    }

    fn top_order(&self, side: OrderSide) -> Option<(OrderSide, Price, &R)> {
        match side {
            OrderSide::Bid => self.bid_side.top_order(),
            OrderSide::Ask => self.ask_side.top_order(),
        }
    }

    // ── Orders Look Up ────────────────────────────────────────────────────────

    fn orders_at(&self, side: OrderSide, price: Price) -> Vec<(OrderSide, Price, &R)> {
        match side {
            OrderSide::Bid => self.bid_side.orders_at(price),
            OrderSide::Ask => self.ask_side.orders_at(price),
        }
    }

    fn orders_up_to(&self, side: OrderSide, bound: Option<Price>) -> Vec<(OrderSide, Price, &R)> {
        match side {
            OrderSide::Bid => self.bid_side.orders_up_to(bound),
            OrderSide::Ask => self.ask_side.orders_up_to(bound),
        }
    }

    fn orders_up_to_quantity(
        &self,
        side: OrderSide,
        bound: Option<Quantity>,
    ) -> Vec<(OrderSide, Price, &R)> {
        match side {
            OrderSide::Bid => self.bid_side.orders_up_to_quantity(bound),
            OrderSide::Ask => self.ask_side.orders_up_to_quantity(bound),
        }
    }

    // ── Aggregation Look Up ───────────────────────────────────────────────────

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
    use types::order::{LimitOrder, RestingLimitOrder, TimeInForce};

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

    fn make_book() -> VecL3OrderBook<LimitOrder, RestingLimitOrder> {
        VecL3OrderBook::<LimitOrder, RestingLimitOrder>::default()
    }

    fn entries(
        orders: Vec<(OrderSide, Price, &RestingLimitOrder)>,
    ) -> Vec<(OrderSide, Price, OrderId, Quantity)> {
        orders
            .into_iter()
            .map(|(side, price, order)| (side, price, order.id(), order.quantity()))
            .collect()
    }

    #[test]
    fn unknown_order_errors() {
        let mut book = make_book();

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
        let mut book = make_book();

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
    fn alternate_resting_order_type() {
        let mut book = VecL3OrderBook::<LimitOrder, LimitOrder>::default();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(
            book.order(1.into())
                .map(|(side, price, order)| (side, price, order.quantity())),
            Some((OrderSide::Bid, 100.into(), 10.into()))
        );
        assert_eq!(
            book.quantity_at(OrderSide::Bid, 100.into()),
            Some(10.into())
        );
    }

    #[test]
    fn add_duplicate_order_id_errors() {
        let mut book = make_book();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(
            book.add_order(make_limit(1, 5, OrderSide::Bid, 105)),
            Err(OrderBookError::OrderIdAlreadyExists { order_id: 1.into() })
        );

        assert_eq!(
            book.order(1.into())
                .map(|(side, price, order)| (side, price, order.quantity())),
            Some((OrderSide::Bid, 100.into(), 10.into()))
        );

        assert_eq!(book.top_price(OrderSide::Bid), Some(100.into()));
        assert_eq!(book.quantity_at(OrderSide::Bid, 105.into()), None);
    }

    #[test]
    fn failed_add_order_does_not_update_index() {
        let mut book = make_book();
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
        let mut book = make_book();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(book.cancel_order(1.into()), Ok(()));
        assert!(book.orders_map.is_empty());
    }

    #[test]
    fn modify_order() {
        let mut book = make_book();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(book.modify_order(1.into(), 23.into()), Ok(()));
        assert_eq!(
            book.order(1.into())
                .map(|(side, price, order)| (side, price, order.quantity())),
            Some((OrderSide::Bid, 100.into(), 23.into()))
        );
    }

    #[test]
    fn fill_order() {
        let mut book = make_book();
        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(book.fill_order(1.into(), 4.into()), Ok(()));
        assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), Some(6.into()));
    }

    #[test]
    fn fully_filled_order_is_removed() {
        let mut book = make_book();
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
        let mut book = make_book();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();

        assert_eq!(
            book.order(1.into()).map(|(side, price, order)| (
                side,
                price,
                order.id(),
                order.quantity()
            )),
            Some((
                OrderSide::Bid,
                Price::new(100),
                OrderId::new(1),
                Quantity::new(10)
            ))
        );
        assert_eq!(book.order(42.into()), None);
    }

    #[test]
    fn top_order() {
        let mut book = make_book();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();
        book.add_order(make_limit(2, 10, OrderSide::Bid, 105))
            .unwrap();
        book.add_order(make_limit(3, 10, OrderSide::Ask, 200))
            .unwrap();
        book.add_order(make_limit(4, 10, OrderSide::Ask, 195))
            .unwrap();

        assert_eq!(
            book.top_order(OrderSide::Bid).map(|(side, price, order)| (
                side,
                price,
                order.id(),
                order.quantity()
            )),
            Some((OrderSide::Bid, 105.into(), 2.into(), 10.into()))
        );
        assert_eq!(
            book.top_order(OrderSide::Ask).map(|(side, price, order)| (
                side,
                price,
                order.id(),
                order.quantity()
            )),
            Some((OrderSide::Ask, 195.into(), 4.into(), 10.into()))
        );
    }

    #[test]
    fn orders_at() {
        let mut book = make_book();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();
        book.add_order(make_limit(2, 10, OrderSide::Bid, 100))
            .unwrap();

        let expected_result = vec![
            (
                OrderSide::Bid,
                Price::new(100),
                OrderId::new(1),
                Quantity::new(10),
            ),
            (
                OrderSide::Bid,
                Price::new(100),
                OrderId::new(2),
                Quantity::new(10),
            ),
        ];
        assert_eq!(
            entries(book.orders_at(OrderSide::Bid, 100.into())),
            expected_result
        );
        assert!(book.orders_at(OrderSide::Bid, 42.into()).is_empty());
    }

    #[test]
    fn orders_up_to() {
        let mut book = make_book();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();
        book.add_order(make_limit(2, 5, OrderSide::Bid, 105))
            .unwrap();
        book.add_order(make_limit(3, 7, OrderSide::Ask, 200))
            .unwrap();
        book.add_order(make_limit(4, 8, OrderSide::Ask, 195))
            .unwrap();

        assert_eq!(
            entries(book.orders_up_to(OrderSide::Bid, None)),
            vec![
                (
                    OrderSide::Bid,
                    Price::new(105),
                    OrderId::new(2),
                    Quantity::new(5)
                ),
                (
                    OrderSide::Bid,
                    Price::new(100),
                    OrderId::new(1),
                    Quantity::new(10)
                ),
            ]
        );
        assert_eq!(
            entries(book.orders_up_to(OrderSide::Ask, None)),
            vec![
                (
                    OrderSide::Ask,
                    Price::new(195),
                    OrderId::new(4),
                    Quantity::new(8)
                ),
                (
                    OrderSide::Ask,
                    Price::new(200),
                    OrderId::new(3),
                    Quantity::new(7)
                ),
            ]
        );
        assert_eq!(
            entries(book.orders_up_to(OrderSide::Ask, Some(195.into()))),
            vec![(
                OrderSide::Ask,
                Price::new(195),
                OrderId::new(4),
                Quantity::new(8)
            )]
        );
        assert!(book
            .orders_up_to(OrderSide::Bid, Some(106.into()))
            .is_empty());
    }

    #[test]
    fn orders_up_to_quantity() {
        let mut book = make_book();

        book.add_order(make_limit(1, 10, OrderSide::Bid, 100))
            .unwrap();
        book.add_order(make_limit(2, 5, OrderSide::Bid, 105))
            .unwrap();
        book.add_order(make_limit(3, 7, OrderSide::Ask, 200))
            .unwrap();
        book.add_order(make_limit(4, 8, OrderSide::Ask, 195))
            .unwrap();

        let bids = vec![
            (
                OrderSide::Bid,
                Price::new(105),
                OrderId::new(2),
                Quantity::new(5),
            ),
            (
                OrderSide::Bid,
                Price::new(100),
                OrderId::new(1),
                Quantity::new(10),
            ),
        ];
        let asks = vec![
            (
                OrderSide::Ask,
                Price::new(195),
                OrderId::new(4),
                Quantity::new(8),
            ),
            (
                OrderSide::Ask,
                Price::new(200),
                OrderId::new(3),
                Quantity::new(7),
            ),
        ];

        assert_eq!(
            entries(book.orders_up_to_quantity(OrderSide::Bid, None)),
            bids
        );
        assert_eq!(
            entries(book.orders_up_to_quantity(OrderSide::Ask, None)),
            asks
        );
        assert_eq!(
            entries(book.orders_up_to_quantity(OrderSide::Ask, Some(8.into()))),
            vec![(
                OrderSide::Ask,
                Price::new(195),
                OrderId::new(4),
                Quantity::new(8)
            )]
        );
        assert_eq!(
            entries(book.orders_up_to_quantity(OrderSide::Ask, Some(12.into()))),
            asks
        );
        assert_eq!(
            entries(book.orders_up_to_quantity(OrderSide::Bid, Some(5.into()))),
            vec![(
                OrderSide::Bid,
                Price::new(105),
                OrderId::new(2),
                Quantity::new(5)
            )]
        );
        assert_eq!(
            entries(book.orders_up_to_quantity(OrderSide::Bid, Some(100.into()))),
            bids
        );
        assert!(make_book()
            .orders_up_to_quantity(OrderSide::Bid, Some(1.into()))
            .is_empty());
    }

    #[test]
    fn top_price() {
        let mut book = make_book();

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

        assert_eq!(make_book().top_price(OrderSide::Bid), None);
    }

    #[test]
    fn quantity_at() {
        let mut book = make_book();

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
        let mut book = make_book();

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
