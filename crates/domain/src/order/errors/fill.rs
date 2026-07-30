use crate::order::fields::{ OrderId, Quantity };
use thiserror::Error;

// ── Fill Error ────────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum FillError {
    #[error("fill quantity {fill_quantity:?} must be less than current {order_quantity:?} for filling on order {order_id:?}")]
    FillExceedsQuantity { order_id: OrderId, order_quantity: Quantity, fill_quantity: Quantity },
}
