use super::level::VecL3BookLevel;

use crate::{
    errors::OrderBookError,
    order::{LimitOrder, OrderId, OrderSide, Price, Quantity},
};

// ── Book Side ─────────────────────────────────────────────────────────────────

pub struct VecL3BookSide {
    pub side: OrderSide,
    levels: Vec<VecL3BookLevel>,
}

impl VecL3BookSide {
    pub fn new(side: OrderSide) -> Self {
        Self {
            side,
            levels: Vec::new(),
        }
    }

    fn find_level_pos(&self, price: Price) -> usize {
        match self.side {
            OrderSide::Ask => self.levels.partition_point(|l| l.price < price),
            OrderSide::Bid => self.levels.partition_point(|l| l.price > price),
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

    fn levels_up_to(&self, bound: Option<Price>) -> &[VecL3BookLevel] {
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

    fn levels_up_to_quantity(&self, target: Option<Quantity>) -> &[VecL3BookLevel] {
        let Some(target) = target else {
            return &self.levels;
        };

        let mut quantity = Quantity::from(0);

        for (i, level) in self.levels.iter().enumerate() {
            quantity += level.total_quantity;

            if quantity >= target {
                return &self.levels[..=i];
            }
        }

        &self.levels
    }
}

// ── Book Mutations ────────────────────────────────────────────────────────────

impl VecL3BookSide {
    pub fn add_order(&mut self, order: LimitOrder) {
        let position = self.get_or_create_level(order.price);
        self.levels[position].add_order(order);
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

// ── Book Look Up ──────────────────────────────────────────────────────────────

impl VecL3BookSide {
    pub fn order(
        &self,
        order_id: OrderId,
        order_price: Price,
    ) -> Result<(OrderSide, Price, Quantity), OrderBookError> {
        let position = self.find_level(order_price)?;

        let quantity = self.levels[position].order(order_id)?;
        Ok((self.side, order_price, quantity))
    }

    pub fn top_order(&self) -> Option<(OrderId, Price, Quantity)> {
        let (order_id, quantity) = self.levels.first()?.top_order()?;

        Some((order_id, self.levels[0].price, quantity))
    }

    pub fn orders_at(&self, price: Price) -> Result<Vec<(OrderId, Quantity)>, OrderBookError> {
        let position = self.find_level(price)?;

        Ok(self.levels[position].orders_at())
    }

    pub fn orders_up_to(&self, bound: Option<Price>) -> Vec<(OrderId, Price, Quantity)> {
        let mut orders = Vec::new();

        for level in self.levels_up_to(bound) {
            for (order_id, quantity) in level.orders_at() {
                orders.push((order_id, level.price, quantity));
            }
        }

        orders
    }

    pub fn orders_up_to_quantity(
        &self,
        bound: Option<Quantity>,
    ) -> Vec<(OrderId, Price, Quantity)> {
        let mut orders = Vec::new();

        for level in self.levels_up_to_quantity(bound) {
            for (order_id, quantity) in level.orders_at() {
                orders.push((order_id, level.price, quantity));
            }
        }

        orders
    }

    pub fn top_price(&self) -> Option<Price> {
        Some(self.levels.first()?.price)
    }

    pub fn quantity_at(&self, price: Price) -> Quantity {
        match self.find_level(price) {
            Ok(position) => self.levels[position].total_quantity,
            Err(_) => 0.into(),
        }
    }

    pub fn quantity_up_to(&self, bound: Option<Price>) -> Quantity {
        let mut total = Quantity::new(0);

        for level in self.levels_up_to(bound) {
            total += level.total_quantity;
        }

        total
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_limit(id: u64, side: OrderSide, price: u64) -> LimitOrder {
        LimitOrder::new(id.into(), 0.into(), 1.into(), side, price.into())
    }

    #[test]
    fn unknown_order_errors() {
        let mut side = VecL3BookSide::new(OrderSide::Ask);

        assert_eq!(
            side.cancel_order(1.into(), 100.into()),
            Err(OrderBookError::PriceLevelNotFound { price: 100.into() })
        );
        assert_eq!(
            side.modify_order(1.into(), 100.into(), 5.into()),
            Err(OrderBookError::PriceLevelNotFound { price: 100.into() })
        );

        side.add_order(make_limit(1, OrderSide::Ask, 100));

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
    }

    #[test]
    fn add_order() {
        let mut asks = VecL3BookSide::new(OrderSide::Ask);
        let mut bids = VecL3BookSide::new(OrderSide::Bid);

        for (id, price) in [(1, 100), (2, 110), (3, 105)] {
            asks.add_order(make_limit(id, OrderSide::Ask, price));
            bids.add_order(make_limit(id, OrderSide::Bid, price));
        }

        let ask_prices: Vec<_> = asks.levels.iter().map(|l| l.price).collect();
        let bid_prices: Vec<_> = bids.levels.iter().map(|l| l.price).collect();

        assert_eq!(ask_prices, vec![100.into(), 105.into(), 110.into()]);
        assert_eq!(bid_prices, vec![110.into(), 105.into(), 100.into()]);
    }

    #[test]
    fn cancel_order() {
        let mut side = VecL3BookSide::new(OrderSide::Ask);
        side.add_order(make_limit(1, OrderSide::Ask, 100));
        side.add_order(make_limit(2, OrderSide::Ask, 100));

        assert_eq!(side.cancel_order(1.into(), 100.into()), Ok(()));
        assert_eq!(side.levels.len(), 1);

        assert_eq!(side.cancel_order(2.into(), 100.into()), Ok(()));
        assert!(side.levels.is_empty());
    }

    #[test]
    fn modify_order() {
        let mut side = VecL3BookSide::new(OrderSide::Ask);
        side.add_order(make_limit(1, OrderSide::Ask, 100));

        assert_eq!(side.modify_order(1.into(), 100.into(), 5.into()), Ok(()));
        assert_eq!(side.quantity_at(100.into()), 5.into());
    }

    #[test]
    fn fill_order() {
        let mut side = VecL3BookSide::new(OrderSide::Ask);
        side.add_order(make_limit(1, OrderSide::Ask, 100));

        assert_eq!(side.fill_order(1.into(), 100.into(), 1.into()), Ok(()));
        assert_eq!(side.quantity_at(100.into()), 0.into());
    }

    #[test]
    fn order() {
        let mut asks = VecL3BookSide::new(OrderSide::Ask);

        asks.add_order(make_limit(1, OrderSide::Ask, 100));

        let expected_result = (OrderSide::Ask, Price::new(100), Quantity::new(1));
        assert_eq!(asks.order(1.into(), 100.into()), Ok(expected_result))
    }

    #[test]
    fn top_order() {
        let mut asks = VecL3BookSide::new(OrderSide::Ask);
        let mut bids = VecL3BookSide::new(OrderSide::Bid);

        asks.add_order(make_limit(1, OrderSide::Ask, 100));
        asks.add_order(make_limit(2, OrderSide::Ask, 105));
        bids.add_order(make_limit(3, OrderSide::Bid, 100));
        bids.add_order(make_limit(4, OrderSide::Bid, 95));

        assert_eq!(asks.top_order(), Some((1.into(), 100.into(), 1.into())));
        assert_eq!(bids.top_order(), Some((3.into(), 100.into(), 1.into())));
        assert_eq!(VecL3BookSide::new(OrderSide::Ask).top_order(), None);
    }

    #[test]
    fn orders_at() {
        let mut asks = VecL3BookSide::new(OrderSide::Ask);

        asks.add_order(make_limit(1, OrderSide::Ask, 100));
        asks.add_order(make_limit(2, OrderSide::Ask, 100));

        let expected_result = vec![(1.into(), 1.into()), (2.into(), 1.into())];
        assert_eq!(asks.orders_at(100.into()), Ok(expected_result));
    }

    #[test]
    fn orders_up_to() {
        let mut asks = VecL3BookSide::new(OrderSide::Ask);
        let mut bids = VecL3BookSide::new(OrderSide::Bid);

        asks.add_order(make_limit(1, OrderSide::Ask, 100));
        asks.add_order(make_limit(2, OrderSide::Ask, 100));
        asks.add_order(make_limit(3, OrderSide::Ask, 105));
        bids.add_order(make_limit(4, OrderSide::Bid, 100));
        bids.add_order(make_limit(5, OrderSide::Bid, 95));

        assert_eq!(
            asks.orders_up_to(None),
            vec![
                (1.into(), 100.into(), 1.into()),
                (2.into(), 100.into(), 1.into()),
                (3.into(), 105.into(), 1.into()),
            ]
        );
        assert_eq!(
            asks.orders_up_to(Some(100.into())),
            vec![
                (1.into(), 100.into(), 1.into()),
                (2.into(), 100.into(), 1.into()),
            ]
        );
        assert!(asks.orders_up_to(Some(99.into())).is_empty());

        assert_eq!(
            bids.orders_up_to(Some(95.into())),
            vec![
                (4.into(), 100.into(), 1.into()),
                (5.into(), 95.into(), 1.into()),
            ]
        );
    }

    #[test]
    fn top_price() {
        let mut asks = VecL3BookSide::new(OrderSide::Ask);
        let mut bids = VecL3BookSide::new(OrderSide::Bid);

        asks.add_order(make_limit(1, OrderSide::Ask, 100));
        asks.add_order(make_limit(2, OrderSide::Ask, 105));
        bids.add_order(make_limit(3, OrderSide::Bid, 100));
        bids.add_order(make_limit(4, OrderSide::Bid, 95));

        assert_eq!(asks.top_price(), Some(100.into()));
        assert_eq!(bids.top_price(), Some(100.into()));
        assert_eq!(VecL3BookSide::new(OrderSide::Ask).top_price(), None);
    }

    #[test]
    fn quantity_at() {
        let mut side = VecL3BookSide::new(OrderSide::Ask);

        side.add_order(make_limit(1, OrderSide::Ask, 100));
        side.add_order(make_limit(2, OrderSide::Ask, 100));

        assert_eq!(side.quantity_at(100.into()), 2.into());
        assert_eq!(side.quantity_at(99.into()), 0.into());
    }

    #[test]
    fn quantity_up_to() {
        let mut asks = VecL3BookSide::new(OrderSide::Ask);

        asks.add_order(make_limit(1, OrderSide::Ask, 100));
        asks.add_order(make_limit(2, OrderSide::Ask, 101));
        asks.add_order(make_limit(3, OrderSide::Ask, 103));

        assert_eq!(asks.quantity_up_to(None), 3.into());
        assert_eq!(asks.quantity_up_to(Some(101.into())), 2.into());
        assert_eq!(asks.quantity_up_to(Some(100.into())), 1.into());
        assert_eq!(asks.quantity_up_to(Some(99.into())), 0.into());

        let mut bids = VecL3BookSide::new(OrderSide::Bid);

        bids.add_order(make_limit(1, OrderSide::Bid, 100));
        bids.add_order(make_limit(2, OrderSide::Bid, 99));
        bids.add_order(make_limit(3, OrderSide::Bid, 97));

        assert_eq!(bids.quantity_up_to(None), 3.into());
        assert_eq!(bids.quantity_up_to(Some(100.into())), 1.into());
        assert_eq!(bids.quantity_up_to(Some(99.into())), 2.into());
        assert_eq!(bids.quantity_up_to(Some(97.into())), 3.into());
        assert_eq!(bids.quantity_up_to(Some(101.into())), 0.into());
    }
}
