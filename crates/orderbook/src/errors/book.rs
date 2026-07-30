use super::level::LevelError;
use super::side::SideError;
use domain::order::{OrderError, OrderId};
use thiserror::Error;

// ── Order Book Error ──────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum OrderBookError {
    #[error(transparent)]
    InvalidOrderOperation(#[from] OrderError),

    #[error(transparent)]
    InvalidLevelOperation(#[from] LevelError),

    #[error(transparent)]
    InvalidSideOperation(#[from] SideError),

    #[error("order {order_id:?} is not in the lookup table")]
    IdNotFound { order_id: OrderId },
}
