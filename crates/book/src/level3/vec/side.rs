use super::{level::VecL3BookLevel, orders::VecRestingOrders};

use crate::level3::common::OrderBookError;

use types::order::{
    fields::{OrderId, OrderSide, Price, Quantity},
    orders::{IncomingOrder, Order},
};

// ── Book Side ─────────────────────────────────────────────────────────────────

pub struct VecL3BookSide<R> {
    side: OrderSide,
    levels: Vec<VecL3BookLevel<R>>,
}

impl<R> VecL3BookSide<R> {
    pub fn new(side: OrderSide) -> Self {
        Self {
            side,
            levels: Vec::new(),
        }
    }

    fn find_level_pos(&self, price: Price) -> usize {
        match self.side {
            OrderSide::Ask => self.levels.partition_point(|level| level.price < price),
            OrderSide::Bid => self.levels.partition_point(|level| level.price > price),
        }
    }

    fn find_level(&self, price: Price) -> Result<usize, OrderBookError> {
        let position = self.find_level_pos(price);

        if position < self.levels.len() && self.levels[position].price == price {
            Ok(position)
        } else {
            Err(OrderBookError::PriceLevelNotFound { price })
        }
    }

    fn get_or_create_level(&mut self, price: Price) -> usize {
        let position = self.find_level_pos(price);

        if position == self.levels.len() || self.levels[position].price != price {
            self.levels.insert(position, VecL3BookLevel::new(price));
        }

        position
    }

    fn levels_through(&self, bound: Option<Price>) -> &[VecL3BookLevel<R>] {
        let end = match (self.side, bound) {
            (OrderSide::Ask, Some(bound)) => self.levels.partition_point(|level| level.price <= bound),
            (OrderSide::Bid, Some(bound)) => self.levels.partition_point(|level| level.price >= bound),
            (_, None) => self.levels.len(),
        };

        &self.levels[..end]
    }

    fn levels_through_quantity(&self, limit: Option<Quantity>) -> &[VecL3BookLevel<R>] {
        let Some(limit) = limit else {
            return &self.levels;
        };

        if limit.is_zero() {
            return &self.levels[..0];
        }

        let mut cumulative = Quantity::new(0);

        for (index, level) in self.levels.iter().enumerate() {
            cumulative = (cumulative + level.total_quantity).unwrap_or(Quantity::new(u64::MAX));

            if cumulative >= limit {
                return &self.levels[..=index];
            }
        }

        &self.levels
    }
}

// ── Book Mutations ────────────────────────────────────────────────────────────

impl<R: Order> VecL3BookSide<R> {
    pub fn insert<I>(&mut self, order: I) -> Result<(), OrderBookError>
    where
        I: IncomingOrder,
        R: From<I>,
    {
        let position = self.get_or_create_level(order.limit());

        self.levels[position].insert(order)
    }

    pub fn remove(&mut self, order_id: OrderId, price: Price) -> Result<(), OrderBookError> {
        let position = self.find_level(price)?;

        self.levels[position].remove(order_id)?;

        if self.levels[position].is_empty() {
            self.levels.remove(position);
        }

        Ok(())
    }

    pub fn reduce(
        &mut self,
        order_id: OrderId,
        price: Price,
        remaining_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let position = self.find_level(price)?;

        self.levels[position].reduce(order_id, remaining_quantity)
    }
}

// ── Peek Orders ───────────────────────────────────────────────────────────────

impl<R: Order> VecL3BookSide<R> {
    pub fn order(&self, order_id: OrderId, price: Price) -> Option<(OrderSide, Price, &R)> {
        let position = self.find_level(price).ok()?;
        let order = self.levels[position].order(order_id)?;

        Some((self.side, price, order))
    }

    pub fn best_order(&self) -> Option<(OrderSide, Price, &R)> {
        let level = self.levels.first()?;
        let order = level.best_order()?;

        Some((self.side, level.price, order))
    }

    pub fn best_order_at(&self, price: Price) -> Option<(OrderSide, Price, &R)> {
        let position = self.find_level(price).ok()?;
        let level = &self.levels[position];
        let order = level.best_order()?;

        Some((self.side, price, order))
    }
}

// ── Order Traversal ───────────────────────────────────────────────────────────

impl<R: Order> VecL3BookSide<R> {
    pub fn orders_at(&self, price: Price) -> VecRestingOrders<'_, R> {
        let levels = match self.find_level(price) {
            Ok(position) => &self.levels[position..=position],
            Err(_) => &self.levels[..0],
        };

        VecRestingOrders::new(self.side, levels)
    }

    pub fn orders_through(&self, bound: Option<Price>) -> VecRestingOrders<'_, R> {
        VecRestingOrders::new(self.side, self.levels_through(bound))
    }

    pub fn orders_through_quantity(&self, limit: Option<Quantity>) -> VecRestingOrders<'_, R> {
        VecRestingOrders::new(self.side, self.levels_through_quantity(limit))
    }
}

// ── Quantity Aggregations ─────────────────────────────────────────────────────

impl<R> VecL3BookSide<R> {
    pub fn quantity_at(&self, price: Price) -> Quantity {
        match self.find_level(price) {
            Ok(position) => self.levels[position].total_quantity,
            Err(_) => Quantity::new(0),
        }
    }

    pub fn quantity_through(&self, bound: Option<Price>) -> Quantity {
        self.levels_through(bound)
            .iter()
            .fold(Quantity::new(0), |total, level| {
                (total + level.total_quantity).unwrap_or(Quantity::new(u64::MAX))
            })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use types::order::{orders::RestingLimitOrder, LimitOrder, TimeInForce};

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

    fn side(side: OrderSide) -> VecL3BookSide<RestingLimitOrder> {
        VecL3BookSide::new(side)
    }

    fn sample(order_side: OrderSide) -> VecL3BookSide<RestingLimitOrder> {
        let mut book_side = side(order_side);

        for (id, quantity, price) in [(1, 5, 100), (2, 7, 100), (3, 8, 105)] {
            book_side.insert(limit(id, quantity, order_side, price)).unwrap();
        }

        book_side
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
        let asks = side(OrderSide::Ask);

        assert_eq!(asks.side, OrderSide::Ask);
        assert!(asks.levels.is_empty());

        let bids = side(OrderSide::Bid);

        assert_eq!(bids.side, OrderSide::Bid);
        assert!(bids.levels.is_empty());
    }

    #[test]
    fn insert() {
        let mut asks = side(OrderSide::Ask);
        let mut bids = side(OrderSide::Bid);

        for (id, price) in [(1, 100), (2, 110), (3, 105)] {
            asks.insert(limit(id, 1, OrderSide::Ask, price)).unwrap();
            bids.insert(limit(id, 1, OrderSide::Bid, price)).unwrap();
        }

        let ask_prices: Vec<_> = asks.levels.iter().map(|level| level.price).collect();
        let bid_prices: Vec<_> = bids.levels.iter().map(|level| level.price).collect();

        assert_eq!(ask_prices, vec![price(100), price(105), price(110)]);
        assert_eq!(bid_prices, vec![price(110), price(105), price(100)]);
    }

    #[test]
    fn remove() {
        let mut ask_side = side(OrderSide::Ask);
        ask_side.insert(limit(1, 5, OrderSide::Ask, 100)).unwrap();
        ask_side.insert(limit(2, 7, OrderSide::Ask, 100)).unwrap();

        assert_eq!(ask_side.remove(1.into(), price(100)), Ok(()));
        assert_eq!(ask_side.quantity_at(price(100)), quantity(7));

        assert_eq!(ask_side.remove(2.into(), price(100)), Ok(()));
        assert!(ask_side.levels.is_empty());

        assert_eq!(
            ask_side.remove(42.into(), price(100)),
            Err(OrderBookError::PriceLevelNotFound { price: price(100) })
        );
    }

    #[test]
    fn reduce() {
        let mut ask_side = side(OrderSide::Ask);
        ask_side.insert(limit(1, 5, OrderSide::Ask, 100)).unwrap();

        assert_eq!(ask_side.reduce(1.into(), price(100), quantity(2)), Ok(()));
        assert_eq!(ask_side.quantity_at(price(100)), quantity(2));

        assert_eq!(
            ask_side.reduce(42.into(), price(100), quantity(1)),
            Err(OrderBookError::OrderIdNotFound { order_id: 42.into() })
        );
    }

    #[test]
    fn order() {
        let asks = sample(OrderSide::Ask);

        assert_eq!(
            asks.order(1.into(), price(100))
                .map(|(side, price, order)| (side, price, order.quantity())),
            Some((OrderSide::Ask, price(100), quantity(5)))
        );
        assert_eq!(asks.order(42.into(), price(100)), None);
        assert_eq!(asks.order(1.into(), price(99)), None);
    }

    #[test]
    fn best_order() {
        let asks = sample(OrderSide::Ask);
        let bids = sample(OrderSide::Bid);

        assert_eq!(
            asks.best_order().map(|(side, price, order)| (side, price, order.id())),
            Some((OrderSide::Ask, price(100), 1.into()))
        );
        assert_eq!(
            bids.best_order().map(|(side, price, order)| (side, price, order.id())),
            Some((OrderSide::Bid, price(105), 3.into()))
        );
        assert_eq!(side(OrderSide::Ask).best_order(), None);
    }

    #[test]
    fn best_order_at() {
        let asks = sample(OrderSide::Ask);

        assert_eq!(
            asks.best_order_at(price(100))
                .map(|(side, price, order)| (side, price, order.id())),
            Some((OrderSide::Ask, price(100), 1.into()))
        );
        assert!(asks.best_order_at(price(99)).is_none());
    }

    #[test]
    fn orders_at() {
        let asks = sample(OrderSide::Ask);

        assert_eq!(
            entries(asks.orders_at(price(100))),
            vec![(OrderSide::Ask, price(100), 1, 5), (OrderSide::Ask, price(100), 2, 7),]
        );
        assert!(asks.orders_at(price(99)).next().is_none());
    }

    #[test]
    fn orders_through() {
        let asks = sample(OrderSide::Ask);
        let bids = sample(OrderSide::Bid);

        assert_eq!(
            entries(asks.orders_through(Some(price(100)))),
            vec![(OrderSide::Ask, price(100), 1, 5), (OrderSide::Ask, price(100), 2, 7),]
        );
        assert_eq!(
            entries(asks.orders_through(None)),
            vec![
                (OrderSide::Ask, price(100), 1, 5),
                (OrderSide::Ask, price(100), 2, 7),
                (OrderSide::Ask, price(105), 3, 8),
            ]
        );
        assert_eq!(
            entries(bids.orders_through(Some(price(100)))),
            vec![
                (OrderSide::Bid, price(105), 3, 8),
                (OrderSide::Bid, price(100), 1, 5),
                (OrderSide::Bid, price(100), 2, 7),
            ]
        );
        assert!(asks.orders_through(Some(price(99))).next().is_none());
        assert!(bids.orders_through(Some(price(106))).next().is_none());
    }

    #[test]
    fn orders_through_quantity() {
        let asks = sample(OrderSide::Ask);
        let bids = sample(OrderSide::Bid);

        assert!(asks.orders_through_quantity(Some(quantity(0))).next().is_none());
        assert_eq!(
            entries(asks.orders_through_quantity(Some(quantity(5)))),
            vec![(OrderSide::Ask, price(100), 1, 5), (OrderSide::Ask, price(100), 2, 7),]
        );
        assert_eq!(
            entries(asks.orders_through_quantity(Some(quantity(12)))),
            vec![(OrderSide::Ask, price(100), 1, 5), (OrderSide::Ask, price(100), 2, 7),]
        );
        assert_eq!(
            entries(asks.orders_through_quantity(None)),
            vec![
                (OrderSide::Ask, price(100), 1, 5),
                (OrderSide::Ask, price(100), 2, 7),
                (OrderSide::Ask, price(105), 3, 8),
            ]
        );
        assert_eq!(
            entries(bids.orders_through_quantity(Some(quantity(8)))),
            vec![(OrderSide::Bid, price(105), 3, 8)]
        );
    }

    #[test]
    fn quantity_at() {
        let asks = sample(OrderSide::Ask);

        assert_eq!(asks.quantity_at(price(100)), quantity(12));
        assert_eq!(asks.quantity_at(price(105)), quantity(8));
        assert_eq!(asks.quantity_at(price(99)), quantity(0));
    }

    #[test]
    fn quantity_through() {
        let asks = sample(OrderSide::Ask);
        let bids = sample(OrderSide::Bid);

        assert_eq!(asks.quantity_through(Some(price(100))), quantity(12));
        assert_eq!(asks.quantity_through(None), quantity(20));
        assert_eq!(asks.quantity_through(Some(price(99))), quantity(0));

        assert_eq!(bids.quantity_through(Some(price(100))), quantity(20));
        assert_eq!(bids.quantity_through(Some(price(105))), quantity(8));
        assert_eq!(bids.quantity_through(Some(price(106))), quantity(0));
        assert_eq!(side(OrderSide::Ask).quantity_through(None), quantity(0));
    }
}
