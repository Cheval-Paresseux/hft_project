use crate::{
    errors::OrderBookError,
    order::{LimitOrder, OrderId, Price, Quantity, RestingOrder},
};

// ── Book Level ────────────────────────────────────────────────────────────────

pub struct VecL3BookLevel {
    pub price: Price,
    orders: Vec<RestingOrder>,
}

impl VecL3BookLevel {
    pub fn new(price: Price) -> Self {
        Self {
            price,
            orders: Vec::new(),
        }
    }

    fn find_position(&self, order_id: OrderId) -> Result<usize, OrderBookError> {
        self.orders
            .iter()
            .position(|o| o.id == order_id)
            .ok_or(OrderBookError::OrderIdNotFound { order_id })
    }

    pub fn add_order(&mut self, order: LimitOrder) {
        self.orders.push(order.into());
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Result<(), OrderBookError> {
        let position = self.find_position(order_id)?;
        self.orders.remove(position);

        Ok(())
    }

    pub fn modify_order(
        &mut self,
        order_id: OrderId,
        new_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let position = self.find_position(order_id)?;
        self.orders[position].quantity = new_quantity;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
}
