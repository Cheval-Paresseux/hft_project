use crate::{
    errors::OrderBookError,
    order::{LimitOrder, OrderId, Price, Quantity, RestingOrder},
};

// ── Book Level ────────────────────────────────────────────────────────────────

pub struct VecL3BookLevel {
    pub price: Price,
    pub total_quantity: Quantity,
    orders: Vec<RestingOrder>,
}

impl VecL3BookLevel {
    pub fn new(price: Price) -> Self {
        Self {
            price,
            total_quantity: 0.into(),
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
        self.total_quantity += order.quantity;
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Result<(), OrderBookError> {
        let position = self.find_position(order_id)?;
        self.total_quantity -= self.orders[position].quantity;
        self.orders.remove(position);

        Ok(())
    }

    pub fn modify_order(
        &mut self,
        order_id: OrderId,
        new_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let position = self.find_position(order_id)?;
        let old_quantity = self.orders[position].quantity;
        let delta = Quantity::new(new_quantity.get().abs_diff(old_quantity.get()));

        if new_quantity >= old_quantity {
            self.total_quantity += delta;
        } else {
            self.total_quantity -= delta;
        }

        self.orders[position].quantity = new_quantity;

        Ok(())
    }

    pub fn fill_order(
        &mut self,
        order_id: OrderId,
        fill_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let position = self.find_position(order_id)?;
        self.orders[position].fill(fill_quantity)?;

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

    fn make_limit(id: u64, quantity: u64) -> LimitOrder {
        LimitOrder::new(
            id.into(),
            0.into(),
            quantity.into(),
            OrderSide::Ask,
            100.into(),
        )
    }

    #[test]
    fn add_order() {
        let mut level = VecL3BookLevel::new(100.into());

        level.add_order(make_limit(1, 10));

        assert_eq!(level.orders.len(), 1);
        assert_eq!(level.orders[0].id, 1.into());
        assert_eq!(level.orders[0].quantity, 10.into());
    }

    #[test]
    fn cancel_order() {
        let mut level = VecL3BookLevel::new(100.into());
        level.add_order(make_limit(1, 10));

        assert_eq!(level.cancel_order(1.into()), Ok(()));
        assert!(level.is_empty());
    }

    #[test]
    fn modify_order() {
        let mut level = VecL3BookLevel::new(100.into());
        level.add_order(make_limit(1, 10));

        assert_eq!(level.modify_order(1.into(), 23.into()), Ok(()));
        assert_eq!(level.orders[0].quantity, 23.into());
    }

    #[test]
    fn unknown_order_errors() {
        let mut level = VecL3BookLevel::new(100.into());

        assert_eq!(
            level.cancel_order(42.into()),
            Err(OrderBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
        assert_eq!(
            level.modify_order(42.into(), 5.into()),
            Err(OrderBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
    }
}
