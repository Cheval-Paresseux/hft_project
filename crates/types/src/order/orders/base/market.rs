use super::stop::StopOrder;

use crate::order::{
    fields::{MarketTimeInForce, OrderId, OrderSide, Quantity, Timestamp},
    orders::{Order, OrderError},
};

// ── Market Order ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MarketOrder {
    id: OrderId,
    quantity: Quantity,

    timestamp: Timestamp,
    side: OrderSide,
    tif: MarketTimeInForce,
}

impl MarketOrder {
    pub fn new(id: OrderId, quantity: Quantity, timestamp: Timestamp, side: OrderSide, tif: MarketTimeInForce) -> Self {
        Self {
            id,
            quantity,
            timestamp,
            side,
            tif,
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn side(&self) -> OrderSide {
        self.side
    }

    pub fn tif(&self) -> MarketTimeInForce {
        self.tif
    }
}

// ── Order Trait ───────────────────────────────────────────────────────────────

impl Order for MarketOrder {
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

// ── From Orders ───────────────────────────────────────────────────────────────

impl From<StopOrder> for MarketOrder {
    fn from(order: StopOrder) -> Self {
        Self {
            id: order.id(),
            quantity: order.quantity(),
            timestamp: order.timestamp(),
            side: order.side(),
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

    fn make_market_order() -> MarketOrder {
        MarketOrder::new(
            OrderId::new(72),
            Quantity::new(10),
            Timestamp::new(100_000),
            OrderSide::Bid,
            MarketTimeInForce::FillOrKill,
        )
    }

    #[test]
    fn new() {
        let order = make_market_order();

        assert_eq!(order.id(), OrderId::new(72));
        assert_eq!(order.quantity(), Quantity::new(10));
        assert_eq!(order.timestamp(), Timestamp::new(100_000));
        assert_eq!(order.side(), OrderSide::Bid);
        assert_eq!(order.tif(), MarketTimeInForce::FillOrKill);
    }

    #[test]
    fn timestamp() {
        let order = make_market_order();

        assert_eq!(order.timestamp(), Timestamp::new(100_000));
    }

    #[test]
    fn side() {
        let order = make_market_order();

        assert_eq!(order.side(), OrderSide::Bid);
    }

    #[test]
    fn tif() {
        let order = make_market_order();

        assert_eq!(order.tif(), MarketTimeInForce::FillOrKill);
    }

    #[test]
    fn id() {
        let order = make_market_order();

        assert_eq!(order.id(), OrderId::new(72));
    }

    #[test]
    fn quantity() {
        let order = make_market_order();

        assert_eq!(order.quantity(), Quantity::new(10));
    }

    #[test]
    fn reduce() {
        let mut order = make_market_order();

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
        let mut order = make_market_order();

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
}
