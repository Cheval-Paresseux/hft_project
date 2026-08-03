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

    pub fn best(&self) -> Option<(OrderId, Price, Quantity)> {
        let (order_id, quantity) = self.levels.first()?.best()?;
        Some((order_id, self.levels[0].price, quantity))
    }

    pub fn best_price(&self) -> Option<Price> {
        Some(self.levels.first()?.price)
    }

    pub fn quantity_at(&self, price: Price) -> Quantity {
        match self.find_level(price) {
            Ok(position) => self.levels[position].total_quantity,
            Err(_) => 0.into(),
        }
    }
}

impl VecL3BookSide {
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
}

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

        self.levels[position].fill_order(order_id, fill_quantity)
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
    fn add_order_keeps_levels_sorted() {
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
    fn orders_share_a_level() {
        let mut side = VecL3BookSide::new(OrderSide::Ask);

        side.add_order(make_limit(1, OrderSide::Ask, 100));
        side.add_order(make_limit(2, OrderSide::Ask, 100));

        assert_eq!(side.levels.len(), 1);
    }

    #[test]
    fn cancel_order_removes_level_when_empty() {
        let mut side = VecL3BookSide::new(OrderSide::Ask);
        side.add_order(make_limit(1, OrderSide::Ask, 100));
        side.add_order(make_limit(2, OrderSide::Ask, 100));

        assert_eq!(side.cancel_order(1.into(), 100.into()), Ok(()));
        assert_eq!(side.levels.len(), 1);

        assert_eq!(side.cancel_order(2.into(), 100.into()), Ok(()));
        assert!(side.levels.is_empty());
    }

    #[test]
    fn best_and_best_price() {
        let mut asks = VecL3BookSide::new(OrderSide::Ask);
        let mut bids = VecL3BookSide::new(OrderSide::Bid);

        asks.add_order(make_limit(1, OrderSide::Ask, 100));
        asks.add_order(make_limit(2, OrderSide::Ask, 105));
        bids.add_order(make_limit(3, OrderSide::Bid, 100));
        bids.add_order(make_limit(4, OrderSide::Bid, 95));

        assert_eq!(asks.best(), Some((1.into(), 100.into(), 1.into())));
        assert_eq!(asks.best_price(), Some(100.into()));
        assert_eq!(bids.best(), Some((3.into(), 100.into(), 1.into())));
        assert_eq!(bids.best_price(), Some(100.into()));

        assert_eq!(VecL3BookSide::new(OrderSide::Ask).best(), None);
        assert_eq!(VecL3BookSide::new(OrderSide::Ask).best_price(), None);
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
    fn fill_order() {
        let mut side = VecL3BookSide::new(OrderSide::Ask);
        side.add_order(make_limit(1, OrderSide::Ask, 100));

        assert_eq!(side.fill_order(1.into(), 100.into(), 1.into()), Ok(()));
        assert_eq!(side.quantity_at(100.into()), 0.into());

        assert_eq!(
            side.fill_order(42.into(), 100.into(), 1.into()),
            Err(OrderBookError::OrderIdNotFound { order_id: 42.into() })
        );
    }

    #[test]
    fn missing_level_or_order_errors() {
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
}
