use crate::order::{
    fields::{MarketTimeInForce, OrderId, OrderSide, Price, Quantity, Timestamp},
    orders::{Order, OrderError},
};

// ── Limit Order ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StopOrder {
    id: OrderId,
    quantity: Quantity,

    timestamp: Timestamp,
    side: OrderSide,
    stop: Price,
    tif: MarketTimeInForce,
}

impl StopOrder {
    pub fn new(
        id: OrderId,
        quantity: Quantity,
        timestamp: Timestamp,
        side: OrderSide,
        stop: Price,
        tif: MarketTimeInForce,
    ) -> Self {
        Self {
            id,
            quantity,
            timestamp,
            side,
            stop,
            tif,
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn side(&self) -> OrderSide {
        self.side
    }

    pub fn stop(&self) -> Price {
        self.stop
    }

    pub fn tif(&self) -> MarketTimeInForce {
        self.tif
    }
}

// ── Order Trait ───────────────────────────────────────────────────────────────

impl Order for StopOrder {
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

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stop_order() -> StopOrder {
        StopOrder::new(
            OrderId::new(72),
            Quantity::new(10),
            Timestamp::new(100_000),
            OrderSide::Bid,
            Price::new(100),
            MarketTimeInForce::FillOrKill,
        )
    }

    #[test]
    fn new() {
        let order = make_stop_order();

        assert_eq!(order.id(), OrderId::new(72));
        assert_eq!(order.quantity(), Quantity::new(10));
        assert_eq!(order.timestamp(), Timestamp::new(100_000));
        assert_eq!(order.side(), OrderSide::Bid);
        assert_eq!(order.stop(), Price::new(100));
        assert_eq!(order.tif(), MarketTimeInForce::FillOrKill);
    }

    #[test]
    fn timestamp() {
        let order = make_stop_order();

        assert_eq!(order.timestamp(), Timestamp::new(100_000));
    }

    #[test]
    fn side() {
        let order = make_stop_order();

        assert_eq!(order.side(), OrderSide::Bid);
    }

    #[test]
    fn stop() {
        let order = make_stop_order();

        assert_eq!(order.stop(), Price::new(100));
    }

    #[test]
    fn tif() {
        let order = make_stop_order();

        assert_eq!(order.tif(), MarketTimeInForce::FillOrKill);
    }

    #[test]
    fn id() {
        let order = make_stop_order();

        assert_eq!(order.id(), OrderId::new(72));
    }

    #[test]
    fn quantity() {
        let order = make_stop_order();

        assert_eq!(order.quantity(), Quantity::new(10));
    }

    #[test]
    fn reduce() {
        let mut order = make_stop_order();

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
        let mut order = make_stop_order();

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
