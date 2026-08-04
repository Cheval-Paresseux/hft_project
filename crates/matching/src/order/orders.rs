use super::fields::{OrderKind, TimeInForce};
use orderbook::order::{LimitOrder, OrderId, OrderSide, Price, Quantity, Timestamp};

// ── Order ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Order {
    pub id: OrderId,
    pub timestamp: Timestamp,
    pub quantity: Quantity,
    pub side: OrderSide,
    pub price: Option<Price>,
    pub kind: OrderKind,
    pub tif: TimeInForce,
}

impl Order {
    pub fn new(
        id: OrderId,
        timestamp: Timestamp,
        quantity: Quantity,
        side: OrderSide,
        price: Option<Price>,
        kind: OrderKind,
        tif: TimeInForce,
    ) -> Self {
        Self {
            id,
            timestamp,
            quantity,
            side,
            price,
            kind,
            tif,
        }
    }
}

impl From<Order> for LimitOrder {
    fn from(val: Order) -> LimitOrder {
        LimitOrder {
            id: val.id,
            timestamp: val.timestamp,
            quantity: val.quantity,
            side: val.side,
            price: val.price.unwrap(),
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
        println!("Order: {} bytes", std::mem::size_of::<Order>());
    }
}
