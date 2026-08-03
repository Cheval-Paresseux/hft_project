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

    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
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
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::OrderSide;

    fn make_limit(id: u64, timestamp: u64, quantity: u64, price: u64) -> LimitOrder {
        LimitOrder::new(
            id.into(),
            timestamp.into(),
            quantity.into(),
            OrderSide::Ask,
            price.into(),
        )
    }

    fn make_side() -> VecL3BookSide {
        VecL3BookSide::new(OrderSide::Ask)
    }

    #[test]
    fn add_order() {
        let mut side = make_side();
        let limit_order = make_limit(0, 0, 10, 100);

        side.add_order(limit_order);

        assert!(side.levels[0].price == 100.into());
    }

    #[test]
    fn cancel_order() {
        let mut side = make_side();
        let limit_order = make_limit(0, 0, 10, 100);

        side.add_order(limit_order);
        let result = side.cancel_order(0.into(), 100.into());

        assert_eq!(result, Ok(()));
        assert!(side.is_empty());
    }

    #[test]
    fn modify_order() {
        let mut side = make_side();
        let limit_order = make_limit(0, 0, 10, 100);

        side.add_order(limit_order);
        let result = side.modify_order(0.into(), 100.into(), 23.into());

        assert_eq!(result, Ok(()));
    }
}
