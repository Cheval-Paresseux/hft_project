use domain::order::{ModificationId, OrderId, OrderSide, Price, Quantity};
use thiserror::Error;

// ── Side Error ────────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SideError {
    #[error("Cancellation of order {order_id:?} is triggered at level {price_level:?} on side {side:?} which is empty")]
    CancelAtEmptyLevel { order_id: OrderId, price_level: Price, side: OrderSide },

    #[error("Modification {modification_id:?} is triggered at level {price_level:?} on side {side:?} which is empty")]
    ModifyAtEmptyLevel { modification_id: ModificationId, price_level: Price, side: OrderSide },

    #[error("Filling with quantity {fill_quantity:?} is triggered at level {price_level:?} on side {side:?} which is empty")]
    FillAtEmptyLevel { fill_quantity: Quantity, price_level: Price, side: OrderSide },
}
