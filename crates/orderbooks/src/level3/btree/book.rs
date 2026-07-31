use std::collections::HashMap;

use crate::{
    errors::OrderBookError,
    level3::traits::L3OrderBook,
    order::{LimitOrder, OrderId, OrderSide, Price, Quantity},
};

pub struct BTreeL3OrderBook {
    orders_map: HashMap<OrderId, (OrderSide, Price, usize)>,
}

impl L3OrderBook for BTreeL3OrderBook {
    fn add_order(&mut self, order: LimitOrder) {}

    fn cancel_order(&mut self, order_id: OrderId) -> Result<(), OrderBookError> {
        Ok(())
    }

    fn modify_order(
        &mut self,
        order_id: OrderId,
        new_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        Ok(())
    }

    fn fill(&mut self, order_id: OrderId, quantity: Quantity) -> Result<(), OrderBookError> {
        Ok(())
    }

    fn best(&self, side: OrderSide) -> Option<(OrderId, Price, Quantity)> {
        None
    }

    fn best_price(&self, side: OrderSide) -> Option<Price> {
        None
    }

    fn quantity_at(&self, side: OrderSide, price: Price) -> Quantity {
        Quantity::new(0)
    }
}
