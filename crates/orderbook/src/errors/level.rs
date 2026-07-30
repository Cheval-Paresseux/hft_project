use domain::order::{ModificationId, OrderId, Price};
use thiserror::Error;

// ── Level Error ───────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LevelError {
    #[error("order {order_id:?} is already booked at level {price:?}")]
    IdAlreadyBooked { order_id: OrderId, price: Price },

    #[error("order {order_id:?} is not booked at level {price:?}")]
    IdNotFound { order_id: OrderId, price: Price },

    #[error(
        "modification {modification_id:?} on order {order_id:?} is not handled by level {price:?}"
    )]
    ModificationNotHandled {
        modification_id: ModificationId,
        order_id: OrderId,
        price: Price,
    },
}
