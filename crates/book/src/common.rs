use types::order::{OrderError, OrderId, Price};

use thiserror::Error;

// ── Order Mutations Policy ────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OrderMutationsPolicy {
    KeepPriority,
    LosePriority,
}

// ── Book Error ────────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BookError {
    #[error(transparent)]
    OrderError(#[from] OrderError),

    #[error("order with id {order_id:?} not found")]
    OrderIdNotFound { order_id: OrderId },

    #[error("order with id {order_id:?} already exists")]
    OrderIdAlreadyExists { order_id: OrderId },

    #[error("price level {price:?} not found")]
    PriceLevelNotFound { price: Price },
}
