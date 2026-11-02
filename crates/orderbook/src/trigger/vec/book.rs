use crate::trigger::common::{TriggerBook, TriggerBookError, TriggerCondition};

use types::order::{
    fields::{OrderId, Quantity},
    orders::Order,
};

// ── Vec Trigger Book ──────────────────────────────────────────────────────────

pub struct VecTriggerBook<T, O> {
    condition: TriggerCondition,
    orders: Vec<(T, O)>,
}

impl<T, O> VecTriggerBook<T, O> {
    pub fn new(condition: TriggerCondition) -> Self {
        Self {
            condition,
            orders: Vec::new(),
        }
    }
}

impl<T: Ord, O: Order> VecTriggerBook<T, O> {
    fn find_threshold_pos(&self, threshold: &T) -> usize {
        match self.condition {
            TriggerCondition::AtOrAbove => self.orders.partition_point(|o| o.0 > *threshold),
            TriggerCondition::AtOrBelow => self.orders.partition_point(|o| o.0 < *threshold),
        }
    }

    fn find_order_pos(&self, order_id: OrderId) -> Result<usize, TriggerBookError> {
        self.orders
            .iter()
            .position(|(_, order)| order.id() == order_id)
            .ok_or(TriggerBookError::OrderIdNotFound { order_id })
    }
}

// ── Trigger Book Trait ────────────────────────────────────────────────────────

impl<T: Ord, O: Order> TriggerBook<T, O> for VecTriggerBook<T, O> {
    // ── Book Mutations ────────────────────────────────────────────────────────

    fn add_order(&mut self, threshold: T, order: O) -> Result<(), TriggerBookError> {
        if self.find_order_pos(order.id()).is_ok() {
            return Err(TriggerBookError::OrderIdAlreadyExists {
                order_id: order.id(),
            });
        }

        let position = self.find_threshold_pos(&threshold);
        self.orders.insert(position, (threshold, order));

        Ok(())
    }

    fn cancel_order(&mut self, order_id: OrderId) -> Result<(), TriggerBookError> {
        let position = self.find_order_pos(order_id)?;
        self.orders.remove(position);

        Ok(())
    }

    fn modify_order(
        &mut self,
        order_id: OrderId,
        new_quantity: Quantity,
    ) -> Result<(), TriggerBookError> {
        let position = self.find_order_pos(order_id)?;
        let (_, order) = &mut self.orders[position];

        order.set_quantity(new_quantity);

        Ok(())
    }

    fn trigger_orders(&mut self, trigger: &T) -> Vec<(T, O)> {
        let end = self.find_threshold_pos(trigger);

        self.orders.drain(..end).collect()
    }

    // ── Get Order ─────────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(&T, &O)> {
        let position = self.find_order_pos(order_id).ok()?;
        self.orders
            .get(position)
            .map(|(threshold, order)| (threshold, order))
    }

    fn top_order(&self) -> Option<(&T, &O)> {
        self.orders
            .first()
            .map(|(threshold, order)| (threshold, order))
    }

    // ── Orders Look Up ────────────────────────────────────────────────────────

    fn orders_at(&self, threshold: &T) -> Vec<(&T, &O)> {
        self.orders
            .iter()
            .filter(|(t, _)| t == threshold)
            .map(|(t, order)| (t, order))
            .collect()
    }

    fn orders_up_to(&self, bound: Option<&T>) -> Vec<(&T, &O)> {
        let end = match bound {
            Some(bound) => self.find_threshold_pos(bound),
            None => self.orders.len(),
        };

        self.orders[..end]
            .iter()
            .map(|(threshold, order)| (threshold, order))
            .collect()
    }

    fn orders_up_to_quantity(&self, bound: Option<Quantity>) -> Vec<(&T, &O)> {
        let Some(bound) = bound else {
            return self
                .orders
                .iter()
                .map(|(threshold, order)| (threshold, order))
                .collect();
        };

        let mut cumulative_quantity = Quantity::from(0);
        let mut result = Vec::new();

        for (threshold, order) in &self.orders {
            if cumulative_quantity >= bound {
                break;
            }

            let Some(next_quantity) = cumulative_quantity + order.quantity() else {
                break;
            };

            cumulative_quantity = next_quantity;
            result.push((threshold, order));
        }

        result
    }

    // ── Threshold Lookups ──────────────────────────────────────────

    fn top_threshold(&self) -> Option<&T> {
        Some(&self.orders.first()?.0)
    }

    fn quantity_at(&self, threshold: &T) -> Option<Quantity> {
        self.orders
            .iter()
            .filter(|(t, _)| t == threshold)
            .try_fold(Quantity::from(0), |total, (_, order)| {
                total + order.quantity()
            })
    }

    fn quantity_up_to(&self, bound: Option<&T>) -> Option<Quantity> {
        let end = match bound {
            Some(bound) => self.find_threshold_pos(bound),
            None => self.orders.len(),
        };

        self.orders[..end]
            .iter()
            .try_fold(Quantity::from(0), |total, (_, order)| {
                total + order.quantity()
            })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
}
