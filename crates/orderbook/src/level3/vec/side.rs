use super::level::VecL3BookLevel;

use crate::level3::common::{add_quantities, OrderBookError};

use types::order::{
    fields::{OrderId, OrderSide, Price, Quantity},
    orders::{IncomingOrder, Order},
};

// ── Book Side ─────────────────────────────────────────────────────────────────

pub struct VecL3BookSide<R> {
    pub side: OrderSide,
    levels: Vec<VecL3BookLevel<R>>,
}

impl<R> VecL3BookSide<R> {
    pub fn new(side: OrderSide) -> Self {
        Self {
            side,
            levels: Vec::new(),
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

    fn find_level_pos(&self, price: Price) -> usize {
        match self.side {
            OrderSide::Ask => self.levels.partition_point(|l| l.price < price),
            OrderSide::Bid => self.levels.partition_point(|l| l.price > price),
        }
    }

    fn get_or_create_level(&mut self, price: Price) -> usize {
        let position = self.find_level_pos(price);

        if position == self.levels.len() || self.levels[position].price != price {
            self.levels.insert(position, VecL3BookLevel::new(price));
        }

        position
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

impl<R> VecL3BookSide<R> {
    fn levels_up_to(&self, bound: Option<Price>) -> &[VecL3BookLevel<R>] {
        let end = match (self.side, bound) {
            (OrderSide::Ask, Some(bound)) => {
                self.levels.partition_point(|level| level.price <= bound)
            }
            (OrderSide::Bid, Some(bound)) => {
                self.levels.partition_point(|level| level.price >= bound)
            }
            (_, None) => self.levels.len(),
        };

        &self.levels[..end]
    }

    fn levels_up_to_quantity(&self, target: Option<Quantity>) -> Option<&[VecL3BookLevel<R>]> {
        let Some(target) = target else {
            return Some(&self.levels);
        };

        let mut quantity = Quantity::from(0);

        for (i, level) in self.levels.iter().enumerate() {
            quantity = (quantity + level.total_quantity)?;

            if quantity >= target {
                return Some(&self.levels[..=i]);
            }
        }

        Some(&self.levels)
    }
}

// ── Book Mutations ────────────────────────────────────────────────────────────

impl<R: Order> VecL3BookSide<R> {
    pub fn add_order<I>(&mut self, order: I) -> Result<(), OrderBookError>
    where
        I: IncomingOrder,
        R: From<I>,
    {
        let position = self.get_or_create_level(order.limit());
        self.levels[position].add_order(order)
    }

    pub fn cancel_order(
        &mut self,
        order_id: OrderId,
        order_price: Price,
    ) -> Result<(), OrderBookError> {
        let position = self.find_level(order_price)?;

        self.levels[position].cancel_order(order_id)?;

        if self.levels[position].is_empty() {
            self.levels.remove(position);
        }

        Ok(())
    }

    pub fn modify_order(
        &mut self,
        order_id: OrderId,
        order_price: Price,
        new_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let position = self.find_level(order_price)?;

        self.levels[position].modify_order(order_id, new_quantity)
    }

    pub fn fill_order(
        &mut self,
        order_id: OrderId,
        order_price: Price,
        fill_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let position = self.find_level(order_price)?;

        self.levels[position].fill_order(order_id, fill_quantity)?;

        if self.levels[position].is_empty() {
            self.levels.remove(position);
        }

        Ok(())
    }
}

// ── Get Order ─────────────────────────────────────────────────────────────────

impl<R: Order> VecL3BookSide<R> {
    pub fn order(&self, order_id: OrderId, order_price: Price) -> Option<(OrderSide, Price, &R)> {
        let position = self.find_level(order_price).ok()?;
        let order = self.levels[position].order(order_id)?;

        Some((self.side, order_price, order))
    }

    pub fn top_order(&self) -> Option<(OrderSide, Price, &R)> {
        let level = self.levels.first()?;
        let order = level.top_order()?;

        Some((self.side, level.price, order))
    }
}

// ── Orders Look Up  ───────────────────────────────────────────────────────────

impl<R: Order> VecL3BookSide<R> {
    pub fn orders_at(&self, price: Price) -> Vec<(OrderSide, Price, &R)> {
        let Ok(position) = self.find_level(price) else {
            return Vec::new();
        };

        self.levels[position]
            .orders_at()
            .iter()
            .map(|order| (self.side, price, order))
            .collect()
    }

    pub fn orders_up_to(&self, bound: Option<Price>) -> Vec<(OrderSide, Price, &R)> {
        self.levels_up_to(bound)
            .iter()
            .flat_map(|level| {
                level
                    .orders_at()
                    .iter()
                    .map(|order| (self.side, level.price, order))
            })
            .collect()
    }

    pub fn orders_up_to_quantity(&self, bound: Option<Quantity>) -> Vec<(OrderSide, Price, &R)> {
        let levels = self.levels_up_to_quantity(bound).unwrap_or(&self.levels);

        levels
            .iter()
            .flat_map(|level| {
                level
                    .orders_at()
                    .iter()
                    .map(|order| (self.side, level.price, order))
            })
            .collect()
    }
}

// ── Aggregation Look Up ───────────────────────────────────────────────────────

impl<R> VecL3BookSide<R> {
    pub fn top_price(&self) -> Option<Price> {
        Some(self.levels.first()?.price)
    }

    pub fn quantity_at(&self, price: Price) -> Option<Quantity> {
        match self.find_level(price) {
            Ok(position) => Some(self.levels[position].total_quantity),
            Err(_) => None,
        }
    }

    pub fn quantity_up_to(&self, bound: Option<Price>) -> Option<Quantity> {
        let mut total = Quantity::new(0);

        for level in self.levels_up_to(bound) {
            total = add_quantities(total, level.total_quantity).ok()?;
        }

        Some(total)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use types::order::{LimitOrder, RestingLimitOrder, TimeInForce};

    fn make_limit(id: u64, side: OrderSide, price: u64) -> LimitOrder {
        LimitOrder::new(
            id.into(),
            1.into(),
            0.into(),
            side,
            price.into(),
            TimeInForce::GoodTillCanceled,
        )
    }

    fn side(side: OrderSide) -> VecL3BookSide<RestingLimitOrder> {
        VecL3BookSide::new(side)
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
    fn new() {
        let asks = side(OrderSide::Ask);
        assert_eq!(asks.side, OrderSide::Ask);
        assert!(asks.levels.is_empty());

        let bids = side(OrderSide::Bid);
        assert_eq!(bids.side, OrderSide::Bid);
        assert!(bids.levels.is_empty());
    }

    #[test]
    fn unknown_order_errors() {
        let mut side = side(OrderSide::Ask);

        assert_eq!(
            side.cancel_order(1.into(), 100.into()),
            Err(OrderBookError::PriceLevelNotFound { price: 100.into() })
        );
        assert_eq!(
            side.modify_order(1.into(), 100.into(), 5.into()),
            Err(OrderBookError::PriceLevelNotFound { price: 100.into() })
        );
        assert_eq!(
            side.fill_order(1.into(), 100.into(), 5.into()),
            Err(OrderBookError::PriceLevelNotFound { price: 100.into() })
        );

        side.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();

        assert_eq!(
            side.cancel_order(42.into(), 100.into()),
            Err(OrderBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
        assert_eq!(
            side.modify_order(42.into(), 100.into(), 5.into()),
            Err(OrderBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
        assert_eq!(
            side.fill_order(42.into(), 100.into(), 5.into()),
            Err(OrderBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
    }

    #[test]
    fn add_order() {
        let mut asks = side(OrderSide::Ask);
        let mut bids = side(OrderSide::Bid);

        for (id, price) in [(1, 100), (2, 110), (3, 105)] {
            asks.add_order(make_limit(id, OrderSide::Ask, price))
                .unwrap();
            bids.add_order(make_limit(id, OrderSide::Bid, price))
                .unwrap();
        }

        let ask_prices: Vec<_> = asks.levels.iter().map(|l| l.price).collect();
        let bid_prices: Vec<_> = bids.levels.iter().map(|l| l.price).collect();

        assert_eq!(ask_prices, vec![100.into(), 105.into(), 110.into()]);
        assert_eq!(bid_prices, vec![110.into(), 105.into(), 100.into()]);
    }

    #[test]
    fn cancel_order() {
        let mut side = side(OrderSide::Ask);
        side.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();
        side.add_order(make_limit(2, OrderSide::Ask, 100)).unwrap();

        assert_eq!(side.cancel_order(1.into(), 100.into()), Ok(()));
        assert_eq!(side.levels.len(), 1);

        assert_eq!(side.cancel_order(2.into(), 100.into()), Ok(()));
        assert!(side.levels.is_empty());
    }

    #[test]
    fn modify_order() {
        let mut side = side(OrderSide::Ask);
        side.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();

        assert_eq!(side.modify_order(1.into(), 100.into(), 5.into()), Ok(()));
        assert_eq!(side.quantity_at(100.into()), Some(5.into()));
    }

    #[test]
    fn fill_order() {
        let mut side = side(OrderSide::Ask);
        side.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();

        assert_eq!(side.fill_order(1.into(), 100.into(), 1.into()), Ok(()));
        assert_eq!(side.quantity_at(100.into()), None);
        assert!(side.levels.is_empty());
    }

    #[test]
    fn order() {
        let mut asks = side(OrderSide::Ask);

        asks.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();

        let expected_result = (OrderSide::Ask, Price::new(100), Quantity::new(1));
        assert_eq!(
            asks.order(1.into(), 100.into())
                .map(|(side, price, order)| (side, price, order.quantity())),
            Some(expected_result)
        );
        assert_eq!(asks.order(1.into(), 99.into()), None);
        assert_eq!(asks.order(42.into(), 100.into()), None);
    }

    #[test]
    fn top_order() {
        let mut asks = side(OrderSide::Ask);
        let mut bids = side(OrderSide::Bid);

        asks.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();
        asks.add_order(make_limit(2, OrderSide::Ask, 105)).unwrap();
        bids.add_order(make_limit(3, OrderSide::Bid, 100)).unwrap();
        bids.add_order(make_limit(4, OrderSide::Bid, 95)).unwrap();

        assert_eq!(
            asks.top_order()
                .map(|(side, price, order)| (side, price, order.id())),
            Some((OrderSide::Ask, 100.into(), 1.into()))
        );
        assert_eq!(
            bids.top_order()
                .map(|(side, price, order)| (side, price, order.id())),
            Some((OrderSide::Bid, 100.into(), 3.into()))
        );
        assert_eq!(side(OrderSide::Ask).top_order(), None);
    }

    #[test]
    fn orders_at() {
        let mut asks = side(OrderSide::Ask);

        asks.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();
        asks.add_order(make_limit(2, OrderSide::Ask, 100)).unwrap();

        let expected_result = vec![
            (
                OrderSide::Ask,
                Price::new(100),
                OrderId::new(1),
                Quantity::new(1),
            ),
            (
                OrderSide::Ask,
                Price::new(100),
                OrderId::new(2),
                Quantity::new(1),
            ),
        ];
        assert_eq!(entries(asks.orders_at(100.into())), expected_result);
        assert!(asks.orders_at(99.into()).is_empty());
    }

    #[test]
    fn orders_up_to() {
        let mut asks = side(OrderSide::Ask);
        let mut bids = side(OrderSide::Bid);

        asks.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();
        asks.add_order(make_limit(2, OrderSide::Ask, 100)).unwrap();
        asks.add_order(make_limit(3, OrderSide::Ask, 105)).unwrap();
        bids.add_order(make_limit(4, OrderSide::Bid, 100)).unwrap();
        bids.add_order(make_limit(5, OrderSide::Bid, 95)).unwrap();

        assert_eq!(
            entries(asks.orders_up_to(None)),
            vec![
                (
                    OrderSide::Ask,
                    Price::new(100),
                    OrderId::new(1),
                    Quantity::new(1)
                ),
                (
                    OrderSide::Ask,
                    Price::new(100),
                    OrderId::new(2),
                    Quantity::new(1)
                ),
                (
                    OrderSide::Ask,
                    Price::new(105),
                    OrderId::new(3),
                    Quantity::new(1)
                ),
            ]
        );
        assert_eq!(
            entries(asks.orders_up_to(Some(100.into()))),
            vec![
                (
                    OrderSide::Ask,
                    Price::new(100),
                    OrderId::new(1),
                    Quantity::new(1)
                ),
                (
                    OrderSide::Ask,
                    Price::new(100),
                    OrderId::new(2),
                    Quantity::new(1)
                ),
            ]
        );
        assert!(asks.orders_up_to(Some(99.into())).is_empty());

        assert_eq!(
            entries(bids.orders_up_to(Some(95.into()))),
            vec![
                (
                    OrderSide::Bid,
                    Price::new(100),
                    OrderId::new(4),
                    Quantity::new(1)
                ),
                (
                    OrderSide::Bid,
                    Price::new(95),
                    OrderId::new(5),
                    Quantity::new(1)
                ),
            ]
        );
    }

    #[test]
    fn orders_up_to_quantity() {
        let mut asks = side(OrderSide::Ask);
        let mut bids = side(OrderSide::Bid);

        asks.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();
        asks.add_order(make_limit(2, OrderSide::Ask, 100)).unwrap();
        asks.add_order(make_limit(3, OrderSide::Ask, 105)).unwrap();
        bids.add_order(make_limit(4, OrderSide::Bid, 100)).unwrap();
        bids.add_order(make_limit(5, OrderSide::Bid, 95)).unwrap();

        assert_eq!(
            entries(asks.orders_up_to_quantity(None)),
            vec![
                (
                    OrderSide::Ask,
                    Price::new(100),
                    OrderId::new(1),
                    Quantity::new(1)
                ),
                (
                    OrderSide::Ask,
                    Price::new(100),
                    OrderId::new(2),
                    Quantity::new(1)
                ),
                (
                    OrderSide::Ask,
                    Price::new(105),
                    OrderId::new(3),
                    Quantity::new(1)
                ),
            ]
        );

        let first_level = vec![
            (
                OrderSide::Ask,
                Price::new(100),
                OrderId::new(1),
                Quantity::new(1),
            ),
            (
                OrderSide::Ask,
                Price::new(100),
                OrderId::new(2),
                Quantity::new(1),
            ),
        ];

        assert_eq!(
            entries(asks.orders_up_to_quantity(Some(1.into()))),
            first_level
        );
        assert_eq!(
            entries(asks.orders_up_to_quantity(Some(2.into()))),
            first_level
        );

        assert_eq!(
            entries(asks.orders_up_to_quantity(Some(3.into()))),
            vec![
                (
                    OrderSide::Ask,
                    Price::new(100),
                    OrderId::new(1),
                    Quantity::new(1)
                ),
                (
                    OrderSide::Ask,
                    Price::new(100),
                    OrderId::new(2),
                    Quantity::new(1)
                ),
                (
                    OrderSide::Ask,
                    Price::new(105),
                    OrderId::new(3),
                    Quantity::new(1)
                ),
            ]
        );
        assert_eq!(
            entries(asks.orders_up_to_quantity(Some(10.into()))),
            entries(asks.orders_up_to_quantity(None))
        );

        assert_eq!(
            entries(bids.orders_up_to_quantity(Some(1.into()))),
            vec![(
                OrderSide::Bid,
                Price::new(100),
                OrderId::new(4),
                Quantity::new(1)
            )]
        );
        assert!(side(OrderSide::Ask)
            .orders_up_to_quantity(Some(1.into()))
            .is_empty());
    }

    #[test]
    fn top_price() {
        let mut asks = side(OrderSide::Ask);
        let mut bids = side(OrderSide::Bid);

        asks.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();
        asks.add_order(make_limit(2, OrderSide::Ask, 105)).unwrap();
        bids.add_order(make_limit(3, OrderSide::Bid, 100)).unwrap();
        bids.add_order(make_limit(4, OrderSide::Bid, 95)).unwrap();

        assert_eq!(asks.top_price(), Some(100.into()));
        assert_eq!(bids.top_price(), Some(100.into()));
        assert_eq!(side(OrderSide::Ask).top_price(), None);
    }

    #[test]
    fn quantity_at() {
        let mut side = side(OrderSide::Ask);

        side.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();
        side.add_order(make_limit(2, OrderSide::Ask, 100)).unwrap();

        assert_eq!(side.quantity_at(100.into()), Some(2.into()));
        assert_eq!(side.quantity_at(99.into()), None);
    }

    #[test]
    fn quantity_up_to() {
        let mut asks = side(OrderSide::Ask);

        asks.add_order(make_limit(1, OrderSide::Ask, 100)).unwrap();
        asks.add_order(make_limit(2, OrderSide::Ask, 101)).unwrap();
        asks.add_order(make_limit(3, OrderSide::Ask, 103)).unwrap();

        assert_eq!(asks.quantity_up_to(None), Some(3.into()));
        assert_eq!(asks.quantity_up_to(Some(101.into())), Some(2.into()));
        assert_eq!(asks.quantity_up_to(Some(100.into())), Some(1.into()));
        assert_eq!(asks.quantity_up_to(Some(99.into())), Some(0.into()));

        let mut bids = side(OrderSide::Bid);

        bids.add_order(make_limit(1, OrderSide::Bid, 100)).unwrap();
        bids.add_order(make_limit(2, OrderSide::Bid, 99)).unwrap();
        bids.add_order(make_limit(3, OrderSide::Bid, 97)).unwrap();

        assert_eq!(bids.quantity_up_to(None), Some(3.into()));
        assert_eq!(bids.quantity_up_to(Some(100.into())), Some(1.into()));
        assert_eq!(bids.quantity_up_to(Some(99.into())), Some(2.into()));
        assert_eq!(bids.quantity_up_to(Some(97.into())), Some(3.into()));
        assert_eq!(bids.quantity_up_to(Some(101.into())), Some(0.into()));
    }
}
