use crate::trigger::common::{TriggerBook, TriggerBookError, TriggerCondition};

use types::order::{
    fields::{OrderId, Quantity},
    orders::Order,
};

// ── Vec Trigger Book ──────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
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

impl<T: Ord, O: Order> TriggerBook for VecTriggerBook<T, O> {
    type Threshold = T;
    type Order = O;

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
    use types::order::{fields::Price, orders::RestingLimitOrder};

    fn make_order(id: u64, quantity: u64) -> RestingLimitOrder {
        RestingLimitOrder::new(id.into(), quantity.into(), 0.into())
    }

    fn price(value: u64) -> Price {
        Price::new(value)
    }

    fn make_book(condition: TriggerCondition) -> VecTriggerBook<Price, RestingLimitOrder> {
        VecTriggerBook::new(condition)
    }

    fn sample_book(condition: TriggerCondition) -> VecTriggerBook<Price, RestingLimitOrder> {
        let mut book = make_book(condition);

        book.add_order(price(100), make_order(1, 5)).unwrap();
        book.add_order(price(105), make_order(2, 8)).unwrap();
        book.add_order(price(100), make_order(3, 7)).unwrap();
        book.add_order(price(95), make_order(4, 10)).unwrap();

        book
    }

    fn ref_entries(orders: &[(&Price, &RestingLimitOrder)]) -> Vec<(u64, u64)> {
        orders
            .iter()
            .map(|(threshold, order)| (threshold.get(), order.id().get()))
            .collect()
    }

    fn owned_entries(orders: &[(Price, RestingLimitOrder)]) -> Vec<(u64, u64)> {
        orders
            .iter()
            .map(|(threshold, order)| (threshold.get(), order.id().get()))
            .collect()
    }

    #[test]
    fn new() {
        let above = make_book(TriggerCondition::AtOrAbove);
        assert_eq!(above.condition, TriggerCondition::AtOrAbove);
        assert!(above.orders.is_empty());

        let below = make_book(TriggerCondition::AtOrBelow);
        assert_eq!(below.condition, TriggerCondition::AtOrBelow);
        assert!(below.orders.is_empty());
    }

    #[test]
    fn add_order() {
        let mut book = make_book(TriggerCondition::AtOrAbove);

        book.add_order(price(105), make_order(1, 10)).unwrap();
        book.add_order(price(100), make_order(2, 5)).unwrap();
        book.add_order(price(100), make_order(3, 7)).unwrap();
        book.add_order(price(95), make_order(4, 1)).unwrap();

        assert_eq!(
            owned_entries(&book.orders),
            vec![(95, 4), (100, 2), (100, 3), (105, 1)]
        );

        assert_eq!(
            book.add_order(price(110), make_order(2, 1)),
            Err(TriggerBookError::OrderIdAlreadyExists { order_id: 2.into() })
        );
        assert_eq!(book.orders.len(), 4);
    }

    #[test]
    fn cancel_order() {
        let mut book = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(book.cancel_order(1.into()), Ok(()));
        assert_eq!(
            owned_entries(&book.orders),
            vec![(95, 4), (100, 3), (105, 2)]
        );

        assert_eq!(
            book.cancel_order(42.into()),
            Err(TriggerBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
    }

    #[test]
    fn modify_order() {
        let mut book = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(book.modify_order(1.into(), 4.into()), Ok(()));
        assert_eq!(book.orders[1].1.quantity(), 4.into());

        assert_eq!(
            book.modify_order(42.into(), 1.into()),
            Err(TriggerBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
    }

    #[test]
    fn trigger_orders() {
        let mut above = sample_book(TriggerCondition::AtOrAbove);

        let triggered = above.trigger_orders(&price(100));
        assert_eq!(owned_entries(&triggered), vec![(95, 4), (100, 1), (100, 3)]);
        assert_eq!(owned_entries(&above.orders), vec![(105, 2)]);
        assert!(above.trigger_orders(&price(90)).is_empty());

        let triggered = above.trigger_orders(&price(110));
        assert_eq!(owned_entries(&triggered), vec![(105, 2)]);

        let mut below = sample_book(TriggerCondition::AtOrBelow);

        let triggered = below.trigger_orders(&price(100));
        assert_eq!(
            owned_entries(&triggered),
            vec![(100, 1), (100, 3), (105, 2)]
        );
        assert_eq!(owned_entries(&below.orders), vec![(95, 4)]);
        assert!(below.trigger_orders(&price(110)).is_empty());

        let triggered = below.trigger_orders(&price(90));
        assert_eq!(owned_entries(&triggered), vec![(95, 4)]);
    }

    #[test]
    fn order() {
        let book = sample_book(TriggerCondition::AtOrAbove);

        let (threshold, order) = book.order(3.into()).unwrap();
        assert_eq!(
            (threshold.get(), order.id().get(), order.quantity().get()),
            (100, 3, 7)
        );

        assert!(book.order(42.into()).is_none());
    }

    #[test]
    fn top_order() {
        let mut above = sample_book(TriggerCondition::AtOrAbove);
        above.add_order(price(95), make_order(5, 1)).unwrap();

        let (threshold, order) = above.top_order().unwrap();
        assert_eq!((threshold.get(), order.id().get()), (95, 4));

        let mut below = sample_book(TriggerCondition::AtOrBelow);
        below.add_order(price(105), make_order(6, 1)).unwrap();

        let (threshold, order) = below.top_order().unwrap();
        assert_eq!((threshold.get(), order.id().get()), (105, 2));

        assert!(make_book(TriggerCondition::AtOrAbove).top_order().is_none());
        assert!(make_book(TriggerCondition::AtOrBelow).top_order().is_none());
    }

    #[test]
    fn orders_at() {
        let book = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(
            ref_entries(&book.orders_at(&price(100))),
            vec![(100, 1), (100, 3)]
        );
        assert_eq!(ref_entries(&book.orders_at(&price(105))), vec![(105, 2)]);
        assert!(book.orders_at(&price(99)).is_empty());
    }

    #[test]
    fn orders_up_to() {
        let above = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(
            ref_entries(&above.orders_up_to(Some(&price(100)))),
            vec![(95, 4), (100, 1), (100, 3)]
        );
        assert!(above.orders_up_to(Some(&price(94))).is_empty());
        assert_eq!(
            ref_entries(&above.orders_up_to(None)),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );

        let below = sample_book(TriggerCondition::AtOrBelow);

        assert_eq!(
            ref_entries(&below.orders_up_to(Some(&price(100)))),
            vec![(100, 1), (100, 3), (105, 2)]
        );
        assert!(below.orders_up_to(Some(&price(110))).is_empty());
        assert_eq!(
            ref_entries(&below.orders_up_to(Some(&price(94)))),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );
        assert_eq!(
            ref_entries(&below.orders_up_to(None)),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );
    }

    #[test]
    fn orders_up_to_quantity() {
        let above = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(
            ref_entries(&above.orders_up_to_quantity(None)),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );
        assert_eq!(
            ref_entries(&above.orders_up_to_quantity(Some(10.into()))),
            vec![(95, 4)]
        );
        assert_eq!(
            ref_entries(&above.orders_up_to_quantity(Some(12.into()))),
            vec![(95, 4), (100, 1)]
        );
        assert_eq!(
            ref_entries(&above.orders_up_to_quantity(Some(22.into()))),
            vec![(95, 4), (100, 1), (100, 3)]
        );
        assert_eq!(
            ref_entries(&above.orders_up_to_quantity(Some(100.into()))),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );

        let below = sample_book(TriggerCondition::AtOrBelow);

        assert_eq!(
            ref_entries(&below.orders_up_to_quantity(None)),
            vec![(105, 2), (100, 1), (100, 3), (95, 4)]
        );
        assert_eq!(
            ref_entries(&below.orders_up_to_quantity(Some(8.into()))),
            vec![(105, 2)]
        );
        assert_eq!(
            ref_entries(&below.orders_up_to_quantity(Some(11.into()))),
            vec![(105, 2), (100, 1)]
        );
        assert_eq!(
            ref_entries(&below.orders_up_to_quantity(Some(20.into()))),
            vec![(105, 2), (100, 1), (100, 3)]
        );
        assert_eq!(
            ref_entries(&below.orders_up_to_quantity(Some(21.into()))),
            vec![(105, 2), (100, 1), (100, 3), (95, 4)]
        );
    }

    #[test]
    fn top_threshold() {
        let above = sample_book(TriggerCondition::AtOrAbove);
        let below = sample_book(TriggerCondition::AtOrBelow);

        assert_eq!(above.top_threshold(), Some(&price(95)));
        assert_eq!(below.top_threshold(), Some(&price(105)));

        assert_eq!(make_book(TriggerCondition::AtOrAbove).top_threshold(), None);
        assert_eq!(make_book(TriggerCondition::AtOrBelow).top_threshold(), None);
    }

    #[test]
    fn quantity_at() {
        let book = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(book.quantity_at(&price(95)), Some(10.into()));
        assert_eq!(book.quantity_at(&price(100)), Some(12.into()));
        assert_eq!(book.quantity_at(&price(105)), Some(8.into()));
        assert_eq!(book.quantity_at(&price(99)), None);
        assert_eq!(
            make_book(TriggerCondition::AtOrAbove).quantity_at(&price(100)),
            None
        );
    }

    #[test]
    fn quantity_up_to() {
        let above = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(above.quantity_up_to(Some(&price(100))), Some(22.into()));
        assert_eq!(above.quantity_up_to(Some(&price(94))), Some(0.into()));
        assert_eq!(above.quantity_up_to(Some(&price(110))), Some(30.into()));
        assert_eq!(above.quantity_up_to(None), Some(30.into()));

        let below = sample_book(TriggerCondition::AtOrBelow);

        assert_eq!(below.quantity_up_to(Some(&price(100))), Some(20.into()));
        assert_eq!(below.quantity_up_to(Some(&price(110))), Some(0.into()));
        assert_eq!(below.quantity_up_to(Some(&price(94))), Some(30.into()));
        assert_eq!(below.quantity_up_to(None), Some(30.into()));

        assert_eq!(
            make_book(TriggerCondition::AtOrBelow).quantity_up_to(None),
            Some(0.into())
        );
    }
}
