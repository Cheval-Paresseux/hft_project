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

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
}

impl VecL3BookLevel {
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
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::OrderSide;

    fn make_limit(id: u64, timestamp: u64, quantity: u64) -> LimitOrder {
        LimitOrder::new(
            id.into(),
            timestamp.into(),
            quantity.into(),
            OrderSide::Ask,
            100.into(),
        )
    }

    fn make_level() -> VecL3BookLevel {
        VecL3BookLevel::new(100.into())
    }

    #[test]
    fn add_order() {
        let mut level = make_level();
        let limit_order = make_limit(0, 0, 10);

        level.add_order(limit_order);

        assert!(level.orders[0].id == 0.into());
    }

    #[test]
    fn cancel_order() {
        let mut level = make_level();
        let limit_order = make_limit(0, 0, 10);

        level.add_order(limit_order);
        let result = level.cancel_order(0.into());

        assert_eq!(result, Ok(()));
        assert!(level.is_empty());
    }

    #[test]
    fn modify_order() {
        let mut level = make_level();
        let limit_order = make_limit(0, 0, 10);

        level.add_order(limit_order);
        let result = level.modify_order(0.into(), 23.into());

        assert_eq!(result, Ok(()));
        assert!(level.orders[0].quantity == 23.into());
    }
}
