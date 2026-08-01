use super::fields::{OrderId, OrderSide, Price, Quantity, Timestamp};

use crate::errors::OrderBookError;

// ── Limit Order ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LimitOrder {
    pub id: OrderId,
    pub timestamp: Timestamp,
    pub quantity: Quantity,
    pub side: OrderSide,
    pub price: Price,
}

impl LimitOrder {
    pub fn new(
        id: OrderId,
        timestamp: Timestamp,
        quantity: Quantity,
        side: OrderSide,
        price: Price,
    ) -> Self {
        Self {
            id,
            timestamp,
            quantity,
            side,
            price,
        }
    }
}

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

    pub fn fill(&mut self, fill_quantity: Quantity) -> Result<(), OrderBookError> {
        if self.quantity < fill_quantity {
            return Err(OrderBookError::FillExceedsOrderQuantity {
                order_id: self.id,
                order_quantity: self.quantity,
                fill_quantity,
            });
        }

        self.quantity -= fill_quantity;

        Ok(())
    }
}

impl From<LimitOrder> for RestingOrder {
    #[inline]
    fn from(order: LimitOrder) -> Self {
        Self {
            id: order.id,
            timestamp: order.timestamp,
            quantity: order.quantity,
        }
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
        println!("Limit Order: {} bytes", std::mem::size_of::<LimitOrder>());

        println!(
            "Resting Order: {} bytes",
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
