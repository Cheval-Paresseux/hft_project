use types::order::{OrderError, OrderId, Price, Quantity};

use thiserror::Error;

// ── Quantity Helpers ──────────────────────────────────────────────────────────

pub(crate) fn add_quantities(lhs: Quantity, rhs: Quantity) -> Quantity {
    (lhs + rhs).unwrap_or(Quantity::new(u64::MAX))
}

pub(crate) fn sub_quantities(lhs: Quantity, rhs: Quantity) -> Quantity {
    (lhs - rhs).unwrap_or(Quantity::new(0))
}

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
