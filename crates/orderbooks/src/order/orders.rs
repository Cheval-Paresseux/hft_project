use crate::order::errors::OrderError;
use crate::order::fields::{OrderId, Quantity, Timestamp};

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
    fn fill() {}
}
