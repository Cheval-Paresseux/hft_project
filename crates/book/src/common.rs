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

    #[error(transparent)]
    LocationError(#[from] LocationError),

    #[error(transparent)]
    MutationError(#[from] MutationError),
}

// ── Location Error ────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LocationError {
    #[error("order with id {order_id:?} not found")]
    IdNotFound { order_id: OrderId },

    #[error("price level {price:?} not found")]
    PriceLevelNotFound { price: Price },
}

// ── Mutation Error ────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MutationError {
    #[error("order with id {order_id:?} already exists")]
    IdAlreadyExists { order_id: OrderId },

    #[error("fill of {filled:?} exceeds quantity {quantity:?} on order {order_id:?}")]
    FillTooLarge {
        order_id: OrderId,
        quantity: Quantity,
        filled: Quantity,
    },
}
