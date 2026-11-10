use crate::order::fields::{OrderId, OrderSide, Price, Quantity};

use thiserror::Error;

// ── Order Error ───────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum OrderError {
    #[error("reduction exceeds current quantity on order {order_id:?}")]
    ReductionTooLarge {
        order_id: OrderId,
        current: Quantity,
        new: Quantity,
    },

    #[error("increase below current quantity on order {order_id:?}")]
    IncreaseTooSmall {
        order_id: OrderId,
        current: Quantity,
        new: Quantity,
    },
}

// ── Order Trait ───────────────────────────────────────────────────────────────

pub trait Order {
    fn id(&self) -> OrderId;
    fn quantity(&self) -> Quantity;

    fn reduce(&mut self, remaining_quantity: Quantity) -> Result<(), OrderError>;
    fn increase(&mut self, remaining_quantity: Quantity) -> Result<(), OrderError>;
}

// ── Incoming Order ────────────────────────────────────────────────────────────

pub trait IncomingOrder: Order {
    fn side(&self) -> OrderSide;
    fn limit(&self) -> Price;
}
