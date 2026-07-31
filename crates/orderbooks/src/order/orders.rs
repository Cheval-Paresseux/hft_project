use super::fields::{OrderId, Quantity, Timestamp};

use crate::errors::OrderError;

// ── Resting Order ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RestingOrder {
    pub id: OrderId,
    pub timestamp: Timestamp,
    pub quantity: Quantity,
}

impl RestingOrder {
    pub fn new(id: OrderId, timestamp: Timestamp, quantity: Quantity) -> Self {
        Self {
            id,
            timestamp,
            quantity,
        }
    }

    pub fn fill(&mut self, fill_quantity: Quantity) -> Result<(), OrderError> {
        if self.quantity < fill_quantity {
            return Err(OrderError::FillExceedsQuantity {
                order_id: self.id,
                order_quantity: self.quantity,
                fill_quantity,
            });
        }

        self.quantity -= fill_quantity;

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_order_size() {
        println!(
            "RestingOrder: {} bytes",
            std::mem::size_of::<RestingOrder>()
        );
    }

    #[test]
    fn fill() {
        let mut order = RestingOrder::new(OrderId::new(0), Timestamp::new(0), Quantity::new(10));

        let fill_quantity: Quantity = 5.into();

        let result = order.fill(fill_quantity);

        assert!(result.is_ok());
        assert_eq!(order.quantity, Quantity::new(5));
    }

    #[test]
    fn fill_exceeds() {
        let mut order = RestingOrder::new(OrderId::new(0), Timestamp::new(0), Quantity::new(10));

        let fill_quantity: Quantity = 15.into();

        let result = order.fill(fill_quantity);

        assert!(result.is_err());
    }
}
