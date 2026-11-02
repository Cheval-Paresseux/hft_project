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
    fn lower_bound(&self, threshold: &T) -> usize {
        self.orders.partition_point(|(t, _)| t < threshold)
    }

    fn upper_bound(&self, threshold: &T) -> usize {
        self.orders.partition_point(|(t, _)| t <= threshold)
    }

    fn trigger_split(&self, trigger: &T) -> usize {
        match self.condition {
            TriggerCondition::AtOrAbove => self
                .orders
                .partition_point(|(threshold, _)| self.condition.is_triggered(threshold, trigger)),

            TriggerCondition::AtOrBelow => self
                .orders
                .partition_point(|(threshold, _)| !self.condition.is_triggered(threshold, trigger)),
        }
    }

    fn trigger_range(&self, trigger: &T) -> (usize, usize) {
        let split = self.trigger_split(trigger);

        match self.condition {
            TriggerCondition::AtOrAbove => (0, split),
            TriggerCondition::AtOrBelow => (split, self.orders.len()),
        }
    }

    fn find_order_pos(&self, order_id: OrderId) -> Result<usize, TriggerBookError> {
        self.orders
            .iter()
            .position(|(_, order)| order.id() == order_id)
            .ok_or(TriggerBookError::OrderIdNotFound { order_id })
    }

    fn top_entry(&self) -> Option<&(T, O)> {
        match self.condition {
            TriggerCondition::AtOrAbove => self.orders.first(),
            TriggerCondition::AtOrBelow => {
                let threshold = &self.orders.last()?.0;
                self.orders.get(self.lower_bound(threshold))
            }
        }
    }

    fn orders_from_top(&self) -> impl Iterator<Item = &(T, O)> {
        let mut index = self.orders.len();
        let mut group_start = index;
        let mut group_end = index;

        std::iter::from_fn(move || {
            if index == group_end {
                if group_start == 0 {
                    return None;
                }

                group_end = group_start;
                let threshold = &self.orders[group_end - 1].0;
                group_start = self.orders[..group_end].partition_point(|(t, _)| t < threshold);
                index = group_start;
            }

            let entry = &self.orders[index];
            index += 1;

            Some(entry)
        })
    }

    fn take_up_to_quantity<'a, I>(
        &'a self,
        orders: I,
        bound: Option<Quantity>,
    ) -> Vec<(&'a T, &'a O)>
    where
        I: Iterator<Item = &'a (T, O)>,
    {
        let Some(bound) = bound else {
            return orders
                .map(|(threshold, order)| (threshold, order))
                .collect();
        };

        let mut cumulative_quantity = Quantity::from(0);
        let mut result = Vec::new();

        for (threshold, order) in orders {
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

        let position = self.upper_bound(&threshold);
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
        let (start, end) = self.trigger_range(trigger);

        self.orders.drain(start..end).collect()
    }

    // ── Get Order ─────────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(&T, &O)> {
        let position = self.find_order_pos(order_id).ok()?;
        self.orders
            .get(position)
            .map(|(threshold, order)| (threshold, order))
    }

    fn top_order(&self) -> Option<(&T, &O)> {
        self.top_entry()
            .map(|(threshold, order)| (threshold, order))
    }

    // ── Orders Look Up ────────────────────────────────────────────────────────

    fn orders_at(&self, threshold: &T) -> Vec<(&T, &O)> {
        let start = self.lower_bound(threshold);

        self.orders[start..]
            .iter()
            .take_while(|(t, _)| t == threshold)
            .map(|(t, order)| (t, order))
            .collect()
    }

    fn orders_up_to(&self, bound: Option<&T>) -> Vec<(&T, &O)> {
        let (start, end) = match bound {
            Some(bound) => self.trigger_range(bound),
            None => (0, self.orders.len()),
        };

        self.orders[start..end]
            .iter()
            .map(|(threshold, order)| (threshold, order))
            .collect()
    }

    fn orders_up_to_quantity(&self, bound: Option<Quantity>) -> Vec<(&T, &O)> {
        match self.condition {
            TriggerCondition::AtOrAbove => self.take_up_to_quantity(self.orders.iter(), bound),
            TriggerCondition::AtOrBelow => self.take_up_to_quantity(self.orders_from_top(), bound),
        }
    }

    // ── Threshold Lookups ──────────────────────────────────────────

    fn top_threshold(&self) -> Option<&T> {
        self.top_entry().map(|(threshold, _)| threshold)
    }

    fn quantity_at(&self, threshold: &T) -> Option<Quantity> {
        let start = self.lower_bound(threshold);
        let mut total = Quantity::from(0);
        let mut found = false;

        for (t, order) in &self.orders[start..] {
            if t != threshold {
                break;
            }

            found = true;
            total = (total + order.quantity())?;
        }

        found.then_some(total)
    }

    fn quantity_up_to(&self, bound: Option<&T>) -> Option<Quantity> {
        let (start, end) = match bound {
            Some(bound) => self.trigger_range(bound),
            None => (0, self.orders.len()),
        };

        self.orders[start..end]
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
