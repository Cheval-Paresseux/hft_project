use super::stop_limit::StopLimitOrder;

use crate::order::{
    fields::{OrderId, OrderSide, Price, Quantity, TimeInForce, Timestamp},
    orders::{IncomingOrder, Order, OrderError},
};

// ── Limit Order ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LimitOrder {
    id: OrderId,
    quantity: Quantity,

    side: OrderSide,
    limit: Price,

    timestamp: Timestamp,
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

    fn reduce(&mut self, remaining_quantity: Quantity) -> Result<(), OrderError> {
        if self.quantity() < remaining_quantity {
            return Err(OrderError::ReductionTooLarge {
                order_id: self.id(),
                current: self.quantity(),
                new: remaining_quantity,
            });
        }

        self.quantity = remaining_quantity;

        Ok(())
    }

    fn increase(&mut self, remaining_quantity: Quantity) -> Result<(), OrderError> {
        if self.quantity() > remaining_quantity {
            return Err(OrderError::IncreaseTooSmall {
                order_id: self.id(),
                current: self.quantity(),
                new: remaining_quantity,
            });
        }

        self.quantity = remaining_quantity;

        Ok(())
    }
}

// ── Incoming Order Trait ──────────────────────────────────────────────────────

impl IncomingOrder for LimitOrder {
    fn side(&self) -> OrderSide {
        self.side
    }

    fn limit(&self) -> Price {
        self.limit
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
    fn new() {
        let order = make_limit_order();

        assert_eq!(order.id(), OrderId::new(72));
        assert_eq!(order.quantity(), Quantity::new(10));
        assert_eq!(order.timestamp(), Timestamp::new(100_000));
        assert_eq!(order.side(), OrderSide::Bid);
        assert_eq!(order.limit(), Price::new(100));
        assert_eq!(order.tif(), TimeInForce::GoodTillCanceled);
    }

    #[test]
    fn timestamp() {
        let order = make_limit_order();

        assert_eq!(order.timestamp(), Timestamp::new(100_000));
    }

    #[test]
    fn tif() {
        let order = make_limit_order();

        assert_eq!(order.tif(), TimeInForce::GoodTillCanceled);
    }

    #[test]
    fn id() {
        let order = make_limit_order();

        assert_eq!(order.id(), OrderId::new(72));
    }

    #[test]
    fn quantity() {
        let order = make_limit_order();

        assert_eq!(order.quantity(), Quantity::new(10));
    }

    #[test]
    fn reduce() {
        let mut order = make_limit_order();

        assert_eq!(order.reduce(Quantity::new(4)), Ok(()));
        assert_eq!(order.quantity(), Quantity::new(4));

        assert_eq!(
            order.reduce(Quantity::new(10)),
            Err(OrderError::ReductionTooLarge {
                order_id: OrderId::new(72),
                current: Quantity::new(4),
                new: Quantity::new(10),
            })
        );
        assert_eq!(order.quantity(), Quantity::new(4));

        assert_eq!(order.reduce(Quantity::new(0)), Ok(()));
        assert_eq!(order.quantity(), Quantity::new(0));
    }

    #[test]
    fn increase() {
        let mut order = make_limit_order();

        assert_eq!(order.increase(Quantity::new(25)), Ok(()));
        assert_eq!(order.quantity(), Quantity::new(25));

        assert_eq!(
            order.increase(Quantity::new(10)),
            Err(OrderError::IncreaseTooSmall {
                order_id: OrderId::new(72),
                current: Quantity::new(25),
                new: Quantity::new(10),
            })
        );
        assert_eq!(order.quantity(), Quantity::new(25));

        assert_eq!(order.increase(Quantity::new(25)), Ok(()));
        assert_eq!(order.quantity(), Quantity::new(25));
    }

    #[test]
    fn side() {
        let order = make_limit_order();

        assert_eq!(order.side(), OrderSide::Bid);
    }

    #[test]
    fn limit() {
        let order = make_limit_order();

        assert_eq!(order.limit(), Price::new(100));
    }
}
