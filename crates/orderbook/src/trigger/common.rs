use types::order::{
    fields::{OrderId, Quantity},
    orders::{Order, OrderError},
};

use thiserror::Error;

use crate::trigger;

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
    pub fn is_triggered<K: Ord>(self, trigger: &K, price: &K) -> bool {
        match self {
            Self::AtOrAbove => trigger <= price,
            Self::AtOrBelow => trigger >= price,
        }
    }
}

// ── Trigger Book ──────────────────────────────────────────────────────────────

pub trait TriggerBook<K: Ord, V: Order> {
    // ── Book Mutations ────────────────────────────────────────────────────────

    fn add_order(&mut self, trigger: K, order: V) -> Result<(), TriggerBookError>;
    fn cancel_order(&mut self, order_id: OrderId) -> Result<(), TriggerBookError>;
    fn modify_order(
        &mut self,
        order_id: OrderId,
        new_quantity: Quantity,
    ) -> Result<(), TriggerBookError>;
    fn trigger_orders(&mut self, trigger: K) -> Result<Vec<V>, TriggerBookError>;

    // ── Orders Look Up ────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(&K, &V)>;
    fn top_order(&self) -> Option<(&K, &V)>;
    fn orders_at(&self, trigger: K) -> Option<Vec<(&K, &V)>>;
    fn orders_up_to(&self, bound: Option<K>) -> Option<Vec<(&K, &V)>>;
    fn orders_up_to_quantity(&self, bound: Option<Quantity>) -> Option<Vec<(&K, &V)>>;

    // ── Trigger Look Up ───────────────────────────────────────────────────────

    fn top_trigger(&self) -> Option<&K>;
    fn quantity_at(&self, trigger: K) -> Option<Quantity>;
    fn quantity_up_to(&self, bound: Option<K>) -> Option<Quantity>;
}
