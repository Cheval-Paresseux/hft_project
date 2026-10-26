use crate::order::fields::{OrderId, Quantity};

use thiserror::Error;

// ── Order Error ───────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum OrderError {
    #[error("fill quantity {fill_quantity:?} > {order_quantity:?} on order {order_id:?}")]
    FillExceedsOrderQuantity {
        order_id: OrderId,
        order_quantity: Quantity,
        fill_quantity: Quantity,
    },
}

// ── Order Trait ───────────────────────────────────────────────────────────────

pub trait Order {
    fn id(&self) -> OrderId;
    fn quantity(&self) -> Quantity;
    fn set_quantity(&mut self, quantity: Quantity);
    fn fill(&mut self, fill_quantity: Quantity) -> Result<(), OrderError>;
}
