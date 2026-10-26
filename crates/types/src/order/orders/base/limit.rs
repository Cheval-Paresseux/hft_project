use super::stop_limit::StopLimitOrder;

use crate::order::{
    fields::{OrderId, OrderSide, Price, Quantity, TimeInForce, Timestamp},
    orders::{Order, OrderError},
};

// ── Limit Order ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LimitOrder {
    id: OrderId,
    quantity: Quantity,

    timestamp: Timestamp,
    side: OrderSide,
    limit: Price,
    tif: TimeInForce,
}

impl LimitOrder {
    pub fn new(
        id: OrderId,
        quantity: Quantity,
        timestamp: Timestamp,
        side: OrderSide,
        limit: Price,
        tif: TimeInForce,
    ) -> Self {
        Self {
            id,
            quantity,
            timestamp,
            side,
            limit,
            tif,
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn side(&self) -> OrderSide {
        self.side
    }

    pub fn limit(&self) -> Price {
        self.limit
    }

    pub fn tif(&self) -> TimeInForce {
        self.tif
    }
}

// ── Order Trait ───────────────────────────────────────────────────────────────

impl Order for LimitOrder {
    fn id(&self) -> OrderId {
        self.id
    }

    fn quantity(&self) -> Quantity {
        self.quantity
    }

    fn set_quantity(&mut self, quantity: Quantity) {
        self.quantity = quantity;
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

impl From<StopLimitOrder> for LimitOrder {
    fn from(order: StopLimitOrder) -> Self {
        Self {
            id: order.id(),
            quantity: order.quantity(),
            timestamp: order.timestamp(),
            side: order.side(),
            limit: order.limit(),
            tif: order.tif(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_limit_order() -> LimitOrder {
        LimitOrder::new(
            OrderId::new(72),
            Quantity::new(10),
            Timestamp::new(100_000),
            OrderSide::Bid,
            Price::new(100),
            TimeInForce::GoodTillCanceled,
        )
    }

    #[test]
    fn getters_return_correct_values() {
        let order = make_limit_order();

        assert_eq!(order.id(), OrderId::new(72));
        assert_eq!(order.quantity(), Quantity::new(10));
        assert_eq!(order.timestamp(), Timestamp::new(100_000));
        assert_eq!(order.side(), OrderSide::Bid);
        assert_eq!(order.limit(), Price::new(100));
        assert_eq!(order.tif(), TimeInForce::GoodTillCanceled);
    }

    #[test]
    fn fill_reduces_quantity() {
        let mut order = make_limit_order();

        assert_eq!(order.fill(Quantity::new(3)), Ok(()));
        assert_eq!(order.quantity(), Quantity::new(7));
    }

    #[test]
    fn fill_exceeding_quantity_returns_error() {
        let mut order = make_limit_order();

        assert!(order.fill(Quantity::new(11)).is_err());
        assert_eq!(order.quantity(), Quantity::new(10));
    }

    #[test]
    fn filling_entire_quantity_succeeds() {
        let mut order = make_limit_order();

        assert_eq!(order.fill(Quantity::new(10)), Ok(()));
        assert_eq!(order.quantity(), Quantity::new(0));
    }
}
