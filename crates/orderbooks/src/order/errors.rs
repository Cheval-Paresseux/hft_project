use super::fields::{OrderId, Quantity};
use thiserror::Error;

// ── Order Error ───────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum OrderError {
    #[error("fill quantity {fill_quantity:?} must be less than current {order_quantity:?} for filling on order {order_id:?}")]
    FillExceedsQuantity {
        order_id: OrderId,
        order_quantity: Quantity,
        fill_quantity: Quantity,
    },
}
