use super::{orders::VecRestingOrders, side::VecL3BookSide};

use crate::common::{BookError, OrderMutationsPolicy};
use crate::level3::common::L3OrderBook;

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

    reduce_policy: OrderMutationsPolicy,
    increase_policy: OrderMutationsPolicy,

    _incoming: PhantomData<I>,
}

impl<I, R> VecL3OrderBook<I, R> {
    #[allow(clippy::new_without_default)]
    pub fn new(reduce_policy: OrderMutationsPolicy, increase_policy: OrderMutationsPolicy) -> Self {
        Self {
            ask_side: VecL3BookSide::new(OrderSide::Ask),
            bid_side: VecL3BookSide::new(OrderSide::Bid),
            orders_map: HashMap::new(),

            reduce_policy,
            increase_policy,

            _incoming: PhantomData,
        }
    }

    fn find_order(&self, order_id: OrderId) -> Result<(OrderSide, Price), BookError> {
        self.orders_map
            .get(&order_id)
            .copied()
            .ok_or(BookError::OrderIdNotFound { order_id })
    }
}

// ── L3 Order Book Trait ───────────────────────────────────────────────────────

impl<I, R> L3OrderBook for VecL3OrderBook<I, R>
where
    I: IncomingOrder,
    R: Order + From<I>,
{
    type Incoming = I;
    type Resting = R;

    type RestingOrders<'a>
        = VecRestingOrders<'a, R>
    where
        Self: 'a;

    // ── Book Mutations ────────────────────────────────────────────────────────

    fn insert(&mut self, order: Self::Incoming) -> Result<(), BookError> {
        let order_id = order.id();

        if self.orders_map.contains_key(&order_id) {
            return Err(BookError::OrderIdAlreadyExists { order_id });
        }

        let side = order.side();
        let price = order.limit();

        match side {
            OrderSide::Bid => self.bid_side.insert(order)?,
            OrderSide::Ask => self.ask_side.insert(order)?,
        }

        self.orders_map.insert(order_id, (side, price));

        Ok(())
    }

    fn remove(&mut self, order_id: OrderId) -> Result<(), BookError> {
        let (side, price) = self.find_order(order_id)?;

        match side {
            OrderSide::Bid => self.bid_side.remove(order_id, price)?,
            OrderSide::Ask => self.ask_side.remove(order_id, price)?,
        }

        self.orders_map.remove(&order_id);

        Ok(())
    }

    fn replace(&mut self, old_order_id: OrderId, new_order: Self::Incoming) -> Result<(), BookError> {
        self.find_order(old_order_id)?;

        let new_order_id = new_order.id();

        if new_order_id != old_order_id && self.orders_map.contains_key(&new_order_id) {
            return Err(BookError::OrderIdAlreadyExists { order_id: new_order_id });
        }

        self.remove(old_order_id)?;
        self.insert(new_order)
    }

    // ── Orders Mutations ──────────────────────────────────────────────────────

    fn reduce(&mut self, order_id: OrderId, remaining_quantity: Quantity) -> Result<(), BookError> {
        let (side, price) = self.find_order(order_id)?;

        match side {
            OrderSide::Bid => self
                .bid_side
                .reduce(&self.reduce_policy, order_id, price, remaining_quantity)?,
            OrderSide::Ask => self
                .ask_side
                .reduce(&self.reduce_policy, order_id, price, remaining_quantity)?,
        }

        if remaining_quantity.is_zero() {
            self.orders_map.remove(&order_id);
        }

        Ok(())
    }

    fn increase(&mut self, order_id: OrderId, remaining_quantity: Quantity) -> Result<(), BookError> {
        let (side, price) = self.find_order(order_id)?;

        match side {
            OrderSide::Bid => self
                .bid_side
                .increase(&self.increase_policy, order_id, price, remaining_quantity),
            OrderSide::Ask => self
                .ask_side
                .increase(&self.increase_policy, order_id, price, remaining_quantity),
        }
    }

    // ── Peek Orders ──────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(OrderSide, Price, &Self::Resting)> {
        let (side, price) = self.find_order(order_id).ok()?;

        match side {
            OrderSide::Bid => self.bid_side.order(order_id, price),
            OrderSide::Ask => self.ask_side.order(order_id, price),
        }
    }

    fn best_order(&self, side: OrderSide) -> Option<(OrderSide, Price, &Self::Resting)> {
        match side {
            OrderSide::Bid => self.bid_side.best_order(),
            OrderSide::Ask => self.ask_side.best_order(),
        }
    }

    fn best_order_at(&self, side: OrderSide, price: Price) -> Option<(OrderSide, Price, &Self::Resting)> {
        match side {
            OrderSide::Bid => self.bid_side.best_order_at(price),
            OrderSide::Ask => self.ask_side.best_order_at(price),
        }
    }

    // ── Order Traversal ──────────────────────────────────────────────────────

    fn orders_at(&self, side: OrderSide, price: Price) -> Self::RestingOrders<'_> {
        match side {
            OrderSide::Bid => self.bid_side.orders_at(price),
            OrderSide::Ask => self.ask_side.orders_at(price),
        }
    }

    fn orders_through(&self, side: OrderSide, bound: Option<Price>) -> Self::RestingOrders<'_> {
        match side {
            OrderSide::Bid => self.bid_side.orders_through(bound),
            OrderSide::Ask => self.ask_side.orders_through(bound),
        }
    }

    fn orders_through_quantity(&self, side: OrderSide, limit: Option<Quantity>) -> Self::RestingOrders<'_> {
        match side {
            OrderSide::Bid => self.bid_side.orders_through_quantity(limit),
            OrderSide::Ask => self.ask_side.orders_through_quantity(limit),
        }
    }

    // ── Quantity Aggregations ────────────────────────────────────────────────

    fn quantity_at(&self, side: OrderSide, price: Price) -> Quantity {
        match side {
            OrderSide::Bid => self.bid_side.quantity_at(price),
            OrderSide::Ask => self.ask_side.quantity_at(price),
        }
    }

    fn quantity_through(&self, side: OrderSide, bound: Option<Price>) -> Quantity {
        match side {
            OrderSide::Bid => self.bid_side.quantity_through(bound),
            OrderSide::Ask => self.ask_side.quantity_through(bound),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use types::order::{LimitOrder, OrderError, TimeInForce, orders::RestingLimitOrder};

    fn price(value: u64) -> Price {
        Price::new(value)
    }

    fn quantity(value: u64) -> Quantity {
        Quantity::new(value)
    }

    fn limit(id: u64, quantity: u64, side: OrderSide, price: u64) -> LimitOrder {
        LimitOrder::new(
            id.into(),
            quantity.into(),
            0.into(),
            side,
            Price::new(price),
            TimeInForce::GoodTillCanceled,
        )
    }

    fn make_book() -> VecL3OrderBook<LimitOrder, RestingLimitOrder> {
        VecL3OrderBook::new(OrderMutationsPolicy::KeepPriority, OrderMutationsPolicy::KeepPriority)
    }

    fn sample_book() -> VecL3OrderBook<LimitOrder, RestingLimitOrder> {
        let mut book = make_book();

        book.insert(limit(1, 5, OrderSide::Bid, 100)).unwrap();
        book.insert(limit(2, 7, OrderSide::Bid, 100)).unwrap();
        book.insert(limit(3, 8, OrderSide::Bid, 105)).unwrap();
        book.insert(limit(4, 6, OrderSide::Ask, 200)).unwrap();
        book.insert(limit(5, 4, OrderSide::Ask, 195)).unwrap();

        book
    }

    fn entries<'a>(
        orders: impl IntoIterator<Item = (OrderSide, Price, &'a RestingLimitOrder)>,
    ) -> Vec<(OrderSide, Price, u64, u64)> {
        orders
            .into_iter()
            .map(|(side, price, order)| (side, price, order.id().get(), order.quantity().get()))
            .collect()
    }

    #[test]
    fn new() {
        let book = make_book();

        assert!(book.best_order(OrderSide::Bid).is_none());
        assert!(book.best_order(OrderSide::Ask).is_none());
        assert!(book.orders_through(OrderSide::Bid, None).next().is_none());
        assert_eq!(book.quantity_through(OrderSide::Bid, None), quantity(0));
    }

    #[test]
    fn insert() {
        let mut book = make_book();

        assert_eq!(book.insert(limit(1, 10, OrderSide::Bid, 100)), Ok(()));
        assert_eq!(book.insert(limit(2, 10, OrderSide::Ask, 200)), Ok(()));

        assert_eq!(book.orders_map[&1.into()], (OrderSide::Bid, price(100)));
        assert_eq!(book.orders_map[&2.into()], (OrderSide::Ask, price(200)));

        assert_eq!(
            book.insert(limit(1, 5, OrderSide::Bid, 105)),
            Err(BookError::OrderIdAlreadyExists { order_id: 1.into() })
        );
        assert_eq!(book.quantity_at(OrderSide::Bid, price(105)), quantity(0));
    }

    #[test]
    fn remove() {
        let mut book = sample_book();

        assert_eq!(book.remove(1.into()), Ok(()));
        assert!(book.order(1.into()).is_none());
        assert_eq!(book.quantity_at(OrderSide::Bid, price(100)), quantity(7));

        assert_eq!(
            book.remove(42.into()),
            Err(BookError::OrderIdNotFound { order_id: 42.into() })
        );
    }

    #[test]
    fn replace() {
        let mut book = sample_book();

        assert_eq!(book.replace(1.into(), limit(1, 3, OrderSide::Bid, 110)), Ok(()));
        assert!(book.order(1.into()).is_some());
        assert_eq!(book.quantity_at(OrderSide::Bid, price(100)), quantity(7));
        assert_eq!(book.quantity_at(OrderSide::Bid, price(110)), quantity(3));

        assert_eq!(
            book.replace(42.into(), limit(6, 1, OrderSide::Bid, 100)),
            Err(BookError::OrderIdNotFound { order_id: 42.into() })
        );

        assert_eq!(
            book.replace(1.into(), limit(2, 1, OrderSide::Bid, 100)),
            Err(BookError::OrderIdAlreadyExists { order_id: 2.into() })
        );
    }

    #[test]
    fn reduce() {
        let mut book = sample_book();

        assert_eq!(
            book.reduce(1.into(), quantity(50)),
            Err(BookError::OrderError(OrderError::ReductionTooLarge {
                order_id: 1.into(),
                current: quantity(5),
                new: quantity(50),
            }))
        );

        assert_eq!(book.reduce(1.into(), quantity(2)), Ok(()));
        assert_eq!(book.quantity_at(OrderSide::Bid, price(100)), quantity(9));

        assert_eq!(
            book.reduce(42.into(), quantity(1)),
            Err(BookError::OrderIdNotFound { order_id: 42.into() })
        );

        assert_eq!(book.reduce(2.into(), quantity(0)), Ok(()));
        assert_eq!(book.reduce(1.into(), quantity(0)), Ok(()));

        assert!(book.best_order_at(OrderSide::Bid, price(100)).is_none());
        assert_eq!(book.quantity_at(OrderSide::Bid, price(100)), quantity(0));
        assert_eq!(
            book.reduce(1.into(), quantity(1)),
            Err(BookError::OrderIdNotFound { order_id: 1.into() })
        );
    }

    #[test]
    fn increase() {
        let mut book = sample_book();

        assert_eq!(book.increase(1.into(), quantity(20)), Ok(()));
        assert_eq!(book.quantity_at(OrderSide::Bid, price(100)), quantity(27));

        assert_eq!(
            book.increase(1.into(), quantity(3)),
            Err(BookError::OrderError(OrderError::IncreaseTooSmall {
                order_id: 1.into(),
                current: quantity(20),
                new: quantity(3),
            }))
        );
        assert_eq!(book.quantity_at(OrderSide::Bid, price(100)), quantity(27));

        assert_eq!(
            book.increase(42.into(), quantity(1)),
            Err(BookError::OrderIdNotFound { order_id: 42.into() })
        );
    }

    #[test]
    fn order() {
        let book = sample_book();

        assert_eq!(
            book.order(1.into())
                .map(|(side, price, order)| (side, price, order.quantity())),
            Some((OrderSide::Bid, price(100), quantity(5)))
        );
        assert_eq!(book.order(42.into()), None);
    }

    #[test]
    fn best_order() {
        let book = sample_book();

        assert_eq!(
            book.best_order(OrderSide::Bid)
                .map(|(side, price, order)| (side, price, order.id())),
            Some((OrderSide::Bid, price(105), 3.into()))
        );
        assert_eq!(
            book.best_order(OrderSide::Ask)
                .map(|(side, price, order)| (side, price, order.id())),
            Some((OrderSide::Ask, price(195), 5.into()))
        );
        assert!(make_book().best_order(OrderSide::Bid).is_none());
    }

    #[test]
    fn best_order_at() {
        let book = sample_book();

        assert_eq!(
            book.best_order_at(OrderSide::Bid, price(100))
                .map(|(side, price, order)| (side, price, order.id())),
            Some((OrderSide::Bid, price(100), 1.into()))
        );
        assert!(book.best_order_at(OrderSide::Bid, price(99)).is_none());
    }

    #[test]
    fn orders_at() {
        let book = sample_book();

        assert_eq!(
            entries(book.orders_at(OrderSide::Bid, price(100))),
            vec![(OrderSide::Bid, price(100), 1, 5), (OrderSide::Bid, price(100), 2, 7),]
        );
        assert!(book.orders_at(OrderSide::Bid, price(99)).next().is_none());
    }

    #[test]
    fn orders_through() {
        let book = sample_book();

        assert_eq!(
            entries(book.orders_through(OrderSide::Bid, Some(price(100)))),
            vec![
                (OrderSide::Bid, price(105), 3, 8),
                (OrderSide::Bid, price(100), 1, 5),
                (OrderSide::Bid, price(100), 2, 7),
            ]
        );
        assert_eq!(
            entries(book.orders_through(OrderSide::Ask, None)),
            vec![(OrderSide::Ask, price(195), 5, 4), (OrderSide::Ask, price(200), 4, 6),]
        );
        assert!(book.orders_through(OrderSide::Bid, Some(price(106))).next().is_none());
    }

    #[test]
    fn orders_through_quantity() {
        let book = sample_book();

        assert!(
            book.orders_through_quantity(OrderSide::Bid, Some(quantity(0)))
                .next()
                .is_none()
        );
        assert_eq!(
            entries(book.orders_through_quantity(OrderSide::Bid, Some(quantity(8)))),
            vec![(OrderSide::Bid, price(105), 3, 8)]
        );
        assert_eq!(
            entries(book.orders_through_quantity(OrderSide::Bid, Some(quantity(13)))),
            vec![
                (OrderSide::Bid, price(105), 3, 8),
                (OrderSide::Bid, price(100), 1, 5),
                (OrderSide::Bid, price(100), 2, 7),
            ]
        );
        assert_eq!(
            entries(book.orders_through_quantity(OrderSide::Ask, None)),
            vec![(OrderSide::Ask, price(195), 5, 4), (OrderSide::Ask, price(200), 4, 6),]
        );
    }

    #[test]
    fn quantity_at() {
        let book = sample_book();

        assert_eq!(book.quantity_at(OrderSide::Bid, price(100)), quantity(12));
        assert_eq!(book.quantity_at(OrderSide::Bid, price(99)), quantity(0));
        assert_eq!(book.quantity_at(OrderSide::Ask, price(195)), quantity(4));
    }

    #[test]
    fn quantity_through() {
        let book = sample_book();

        assert_eq!(book.quantity_through(OrderSide::Bid, Some(price(100))), quantity(20));
        assert_eq!(book.quantity_through(OrderSide::Bid, None), quantity(20));
        assert_eq!(book.quantity_through(OrderSide::Bid, Some(price(106))), quantity(0));
        assert_eq!(book.quantity_through(OrderSide::Ask, Some(price(195))), quantity(4));
        assert_eq!(book.quantity_through(OrderSide::Ask, None), quantity(10));
        assert_eq!(make_book().quantity_through(OrderSide::Ask, None), quantity(0));
    }
}
