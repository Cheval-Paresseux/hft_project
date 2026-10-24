use crate::order::{
    fields::{OrderId, Quantity, Timestamp},
    orders::{base::LimitOrder, Order, OrderError},
};

// ── Resting Limit Order ───────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RestingLimitOrder {
    id: OrderId,
    quantity: Quantity,

    timestamp: Timestamp,
}

impl RestingLimitOrder {
    pub fn new(id: OrderId, quantity: Quantity, timestamp: Timestamp) -> Self {
        Self {
            id,
            quantity,
            timestamp,
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}

// ── Order Trait ───────────────────────────────────────────────────────────────

impl Order for RestingLimitOrder {
    fn id(&self) -> OrderId {
        self.id
    }

    fn quantity(&self) -> Quantity {
        self.quantity
    }

    fn fill(&mut self, fill_quantity: Quantity) -> Result<(), OrderError> {
        let order_quantity = self.quantity();

        let new_quantity =
            (order_quantity - fill_quantity).ok_or(OrderError::FillExceedsOrderQuantity {
                order_id: self.id(),
                order_quantity,
                fill_quantity,
            })?;

        self.quantity = new_quantity;

        Ok(())
    }
}

// ── From Orders ───────────────────────────────────────────────────────────────

impl From<LimitOrder> for RestingLimitOrder {
    fn from(order: LimitOrder) -> Self {
        Self {
            id: order.id(),
            quantity: order.quantity(),
            timestamp: order.timestamp(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_resting_limit_order() -> RestingLimitOrder {
        RestingLimitOrder::new(OrderId::new(72), Quantity::new(10), Timestamp::new(100_000))
    }

    #[test]
    fn getters_return_correct_values() {
        let order = make_resting_limit_order();

        assert_eq!(order.id(), OrderId::new(72));
        assert_eq!(order.quantity(), Quantity::new(10));
        assert_eq!(order.timestamp(), Timestamp::new(100_000));
    }

    #[test]
    fn fill_reduces_quantity() {
        let mut order = make_resting_limit_order();

        assert_eq!(order.fill(Quantity::new(3)), Ok(()));
        assert_eq!(order.quantity(), Quantity::new(7));
    }

    #[test]
    fn fill_exceeding_quantity_returns_error() {
        let mut order = make_resting_limit_order();

        assert!(order.fill(Quantity::new(11)).is_err());
        assert_eq!(order.quantity(), Quantity::new(10));
    }

    #[test]
    fn filling_entire_quantity_succeeds() {
        let mut order = make_resting_limit_order();

        assert_eq!(order.fill(Quantity::new(10)), Ok(()));
        assert_eq!(order.quantity(), Quantity::new(0));
    }
}
