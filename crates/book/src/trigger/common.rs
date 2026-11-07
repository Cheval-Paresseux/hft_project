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

    type Orders<'a>: Iterator<Item = (&'a Self::Threshold, &'a Self::Order)>
    where
        Self: 'a;

    fn condition(&self) -> TriggerCondition;

    // ── Book Mutations ────────────────────────────────────────────────────────

    fn insert(&mut self, threshold: Self::Threshold, order: Self::Order) -> Result<(), TriggerBookError>;
    fn remove(&mut self, order_id: OrderId) -> Result<(), TriggerBookError>;

    fn reduce(&mut self, order_id: OrderId, remaining_quantity: Quantity) -> Result<(), TriggerBookError>;
    fn replace(
        &mut self,
        old_order_id: OrderId,
        new_threshold: Self::Threshold,
        new_order: Self::Order,
    ) -> Result<(), TriggerBookError>;

    // ── Trigger Evaluation ───────────────────────────────────────────────────

    fn trigger(&mut self, trigger: Self::Threshold) -> Vec<(Self::Threshold, Self::Order)>;

    // ── Peek Orders ──────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(&Self::Threshold, &Self::Order)>;
    fn best_order(&self) -> Option<(&Self::Threshold, &Self::Order)>;
    fn best_order_at(&self, threshold: &Self::Threshold) -> Option<(&Self::Threshold, &Self::Order)>;

    // ── Order Traversal ──────────────────────────────────────────────────────

    fn orders_at(&self, threshold: &Self::Threshold) -> Self::Orders<'_>;
    fn orders_through(&self, bound: Option<&Self::Threshold>) -> Self::Orders<'_>;
    fn orders_through_quantity(&self, limit: Option<Quantity>) -> Self::Orders<'_>;

    // ── Quantity Aggregations ────────────────────────────────────────────────

    fn quantity_at(&self, threshold: &Self::Threshold) -> Quantity;
    fn quantity_through(&self, bound: Option<&Self::Threshold>) -> Quantity;
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_triggered() {
        let threshold = 100;

        assert!(TriggerCondition::AtOrAbove.is_triggered(&threshold, &100));
        assert!(TriggerCondition::AtOrAbove.is_triggered(&threshold, &101));
        assert!(!TriggerCondition::AtOrAbove.is_triggered(&threshold, &99));

        assert!(TriggerCondition::AtOrBelow.is_triggered(&threshold, &100));
        assert!(TriggerCondition::AtOrBelow.is_triggered(&threshold, &99));
        assert!(!TriggerCondition::AtOrBelow.is_triggered(&threshold, &101));
    }
}
