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
