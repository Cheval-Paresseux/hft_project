use types::order::{
    fields::{OrderId, Quantity},
    orders::{Order, OrderError},
};

use thiserror::Error;

// ── Trigger Book Error ────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TriggerBookError {
    #[error(transparent)]
    OrderError(#[from] OrderError),

    #[error("order with id {order_id:?} not found")]
    OrderIdNotFound { order_id: OrderId },

    #[error("order with id {order_id:?} already exists")]
    OrderIdAlreadyExists { order_id: OrderId },
}

// ── Trigger Condition ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TriggerCondition {
    AtOrAbove,
    AtOrBelow,
}

impl TriggerCondition {
    pub fn is_triggered<T: Ord>(self, threshold: &T, trigger: &T) -> bool {
        match self {
            Self::AtOrAbove => trigger >= threshold,
            Self::AtOrBelow => trigger <= threshold,
        }
    }
}

// ── Trigger Book ──────────────────────────────────────────────────────────────

pub trait TriggerBook<T: Ord, O: Order> {
    // ── Book Mutations ────────────────────────────────────────────────────────
    fn add_order(&mut self, threshold: T, order: O) -> Result<(), TriggerBookError>;
    fn cancel_order(&mut self, order_id: OrderId) -> Result<(), TriggerBookError>;
    fn modify_order(
        &mut self,
        order_id: OrderId,
        new_quantity: Quantity,
    ) -> Result<(), TriggerBookError>;
    fn trigger_orders(&mut self, trigger: &T) -> Vec<(T, O)>;

    // ── Get Order ─────────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(&T, &O)>;
    fn top_order(&self) -> Option<(&T, &O)>;

    // ── Orders Look Up ────────────────────────────────────────────────────────

    fn orders_at(&self, threshold: &T) -> Vec<(&T, &O)>;
    fn orders_up_to(&self, bound: Option<&T>) -> Vec<(&T, &O)>;
    fn orders_up_to_quantity(&self, bound: Option<Quantity>) -> Vec<(&T, &O)>;

    // ── Threshold Lookups ──────────────────────────────────────────

    fn top_threshold(&self) -> Option<&T>;
    fn quantity_at(&self, threshold: &T) -> Option<Quantity>;
    fn quantity_up_to(&self, bound: Option<&T>) -> Option<Quantity>;
}
