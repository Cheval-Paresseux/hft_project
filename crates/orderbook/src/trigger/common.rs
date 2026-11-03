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

pub trait TriggerBook {
    type Threshold: Ord;
    type Order: Order;

    // ── Book Mutations ────────────────────────────────────────────────────────
    fn add_order(
        &mut self,
        threshold: Self::Threshold,
        order: Self::Order,
    ) -> Result<(), TriggerBookError>;
    fn cancel_order(&mut self, order_id: OrderId) -> Result<(), TriggerBookError>;
    fn modify_order(
        &mut self,
        order_id: OrderId,
        new_quantity: Quantity,
    ) -> Result<(), TriggerBookError>;
    fn trigger_orders(&mut self, trigger: &Self::Threshold) -> Vec<(Self::Threshold, Self::Order)>;

    // ── Get Order ─────────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(&Self::Threshold, &Self::Order)>;
    fn top_order(&self) -> Option<(&Self::Threshold, &Self::Order)>;

    // ── Orders Look Up ────────────────────────────────────────────────────────

    fn orders_at(&self, threshold: &Self::Threshold) -> Vec<(&Self::Threshold, &Self::Order)>;
    fn orders_up_to(
        &self,
        bound: Option<&Self::Threshold>,
    ) -> Vec<(&Self::Threshold, &Self::Order)>;
    fn orders_up_to_quantity(
        &self,
        bound: Option<Quantity>,
    ) -> Vec<(&Self::Threshold, &Self::Order)>;

    // ── Threshold Lookups ─────────────────────────────────────────────────────

    fn top_threshold(&self) -> Option<&Self::Threshold>;
    fn quantity_at(&self, threshold: &Self::Threshold) -> Option<Quantity>;
    fn quantity_up_to(&self, bound: Option<&Self::Threshold>) -> Option<Quantity>;
}
