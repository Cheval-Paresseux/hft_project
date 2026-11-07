use super::orders::VecOrders;

use crate::trigger::common::{TriggerBook, TriggerBookError, TriggerCondition};

use types::order::{
    fields::{OrderId, Quantity},
    orders::Order,
};

// ── Helper on Quantity ────────────────────────────────────────────────────────

fn add_quantities(lhs: Quantity, rhs: Quantity) -> Quantity {
    (lhs + rhs).unwrap_or(Quantity::new(u64::MAX))
}

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
}

// ── Trigger Book Trait ────────────────────────────────────────────────────────

impl<T: Ord, O: Order> TriggerBook for VecTriggerBook<T, O> {
    type Threshold = T;
    type Order = O;

    type Orders<'a>
        = VecOrders<'a, T, O>
    where
        Self: 'a;

    fn condition(&self) -> TriggerCondition {
        self.condition
    }

    // ── Book Mutations ────────────────────────────────────────────────────────

    fn insert(&mut self, threshold: T, order: O) -> Result<(), TriggerBookError> {
        if self.find_order_pos(order.id()).is_ok() {
            return Err(TriggerBookError::OrderIdAlreadyExists { order_id: order.id() });
        }

        let position = self.upper_bound(&threshold);
        self.orders.insert(position, (threshold, order));

        Ok(())
    }

    fn remove(&mut self, order_id: OrderId) -> Result<(), TriggerBookError> {
        let position = self.find_order_pos(order_id)?;
        self.orders.remove(position);

        Ok(())
    }

    fn reduce(&mut self, order_id: OrderId, remaining_quantity: Quantity) -> Result<(), TriggerBookError> {
        let position = self.find_order_pos(order_id)?;
        let (_, order) = &mut self.orders[position];

        order.set_quantity(remaining_quantity);

        Ok(())
    }

    fn replace(&mut self, old_order_id: OrderId, new_threshold: T, new_order: O) -> Result<(), TriggerBookError> {
        let old_position = self.find_order_pos(old_order_id)?;

        if new_order.id() != old_order_id && self.find_order_pos(new_order.id()).is_ok() {
            return Err(TriggerBookError::OrderIdAlreadyExists {
                order_id: new_order.id(),
            });
        }

        self.orders.remove(old_position);

        let new_position = self.upper_bound(&new_threshold);
        self.orders.insert(new_position, (new_threshold, new_order));

        Ok(())
    }

    // ── Trigger Evaluation ───────────────────────────────────────────────────

    fn trigger(&mut self, trigger: Self::Threshold) -> Vec<(Self::Threshold, Self::Order)> {
        let (start, end) = self.trigger_range(&trigger);

        self.orders.drain(start..end).collect()
    }

    // ── Peek Orders ──────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(&Self::Threshold, &Self::Order)> {
        let position = self.find_order_pos(order_id).ok()?;

        self.orders.get(position).map(|(threshold, order)| (threshold, order))
    }

    fn best_order(&self) -> Option<(&Self::Threshold, &Self::Order)> {
        self.top_entry().map(|(threshold, order)| (threshold, order))
    }

    fn best_order_at(&self, threshold: &Self::Threshold) -> Option<(&Self::Threshold, &Self::Order)> {
        let position = self.lower_bound(threshold);

        self.orders
            .get(position)
            .filter(|(t, _)| t == threshold)
            .map(|(threshold, order)| (threshold, order))
    }

    // ── Order Traversal ──────────────────────────────────────────────────────

    fn orders_at(&self, threshold: &Self::Threshold) -> Self::Orders<'_> {
        let start = self.lower_bound(threshold);
        let end = self.upper_bound(threshold);

        VecOrders::forward(&self.orders[start..end], None)
    }

    fn orders_through(&self, bound: Option<&Self::Threshold>) -> Self::Orders<'_> {
        let (start, end) = match bound {
            Some(bound) => self.trigger_range(bound),
            None => (0, self.orders.len()),
        };

        VecOrders::forward(&self.orders[start..end], None)
    }

    fn orders_through_quantity(&self, limit: Option<Quantity>) -> Self::Orders<'_> {
        match self.condition {
            TriggerCondition::AtOrAbove => VecOrders::forward(&self.orders, limit),
            TriggerCondition::AtOrBelow => VecOrders::reverse(&self.orders, limit),
        }
    }

    // ── Quantity Aggregations ────────────────────────────────────────────────

    fn quantity_at(&self, threshold: &Self::Threshold) -> Quantity {
        let start = self.lower_bound(threshold);
        let end = self.upper_bound(threshold);

        self.orders[start..end]
            .iter()
            .fold(Quantity::new(0), |total, (_, order)| {
                add_quantities(total, order.quantity())
            })
    }

    fn quantity_through(&self, bound: Option<&Self::Threshold>) -> Quantity {
        self.orders_through(bound).fold(Quantity::new(0), |total, (_, order)| {
            add_quantities(total, order.quantity())
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

        book.insert(price(100), make_order(1, 5)).unwrap();
        book.insert(price(105), make_order(2, 8)).unwrap();
        book.insert(price(100), make_order(3, 7)).unwrap();
        book.insert(price(95), make_order(4, 10)).unwrap();

        book
    }

    fn ref_entries<'a>(orders: impl IntoIterator<Item = (&'a Price, &'a RestingLimitOrder)>) -> Vec<(u64, u64)> {
        orders
            .into_iter()
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
    fn insert() {
        let mut book = make_book(TriggerCondition::AtOrAbove);

        book.insert(price(105), make_order(1, 10)).unwrap();
        book.insert(price(100), make_order(2, 5)).unwrap();
        book.insert(price(100), make_order(3, 7)).unwrap();
        book.insert(price(95), make_order(4, 1)).unwrap();

        assert_eq!(owned_entries(&book.orders), vec![(95, 4), (100, 2), (100, 3), (105, 1)]);

        assert_eq!(
            book.insert(price(110), make_order(2, 1)),
            Err(TriggerBookError::OrderIdAlreadyExists { order_id: 2.into() })
        );
        assert_eq!(book.orders.len(), 4);
    }

    #[test]
    fn remove() {
        let mut book = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(book.remove(1.into()), Ok(()));
        assert_eq!(owned_entries(&book.orders), vec![(95, 4), (100, 3), (105, 2)]);

        assert_eq!(
            book.remove(42.into()),
            Err(TriggerBookError::OrderIdNotFound { order_id: 42.into() })
        );
    }

    #[test]
    fn reduce() {
        let mut book = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(book.reduce(1.into(), 2.into()), Ok(()));
        assert_eq!(book.orders[1].1.quantity(), 2.into());

        assert_eq!(
            book.reduce(42.into(), 1.into()),
            Err(TriggerBookError::OrderIdNotFound { order_id: 42.into() })
        );
    }

    #[test]
    fn replace() {
        let mut book = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(book.replace(1.into(), price(110), make_order(5, 3)), Ok(()));
        assert_eq!(owned_entries(&book.orders), vec![(95, 4), (100, 3), (105, 2), (110, 5)]);

        assert_eq!(
            book.replace(42.into(), price(100), make_order(6, 1)),
            Err(TriggerBookError::OrderIdNotFound { order_id: 42.into() })
        );

        assert_eq!(
            book.replace(3.into(), price(100), make_order(4, 1)),
            Err(TriggerBookError::OrderIdAlreadyExists { order_id: 4.into() })
        );
    }

    #[test]
    fn trigger() {
        let mut above = sample_book(TriggerCondition::AtOrAbove);

        let triggered = above.trigger(price(100));
        assert_eq!(owned_entries(&triggered), vec![(95, 4), (100, 1), (100, 3)]);
        assert_eq!(owned_entries(&above.orders), vec![(105, 2)]);
        assert!(above.trigger(price(90)).is_empty());

        let triggered = above.trigger(price(110));
        assert_eq!(owned_entries(&triggered), vec![(105, 2)]);

        let mut below = sample_book(TriggerCondition::AtOrBelow);

        let triggered = below.trigger(price(100));
        assert_eq!(owned_entries(&triggered), vec![(100, 1), (100, 3), (105, 2)]);
        assert_eq!(owned_entries(&below.orders), vec![(95, 4)]);
        assert!(below.trigger(price(110)).is_empty());

        let triggered = below.trigger(price(90));
        assert_eq!(owned_entries(&triggered), vec![(95, 4)]);
    }

    #[test]
    fn order() {
        let book = sample_book(TriggerCondition::AtOrAbove);

        let (threshold, order) = book.order(3.into()).unwrap();
        assert_eq!((threshold.get(), order.id().get(), order.quantity().get()), (100, 3, 7));

        assert!(book.order(42.into()).is_none());
    }

    #[test]
    fn best_order() {
        let mut above = sample_book(TriggerCondition::AtOrAbove);
        above.insert(price(95), make_order(5, 1)).unwrap();

        let (threshold, order) = above.best_order().unwrap();
        assert_eq!((threshold.get(), order.id().get()), (95, 4));

        let mut below = sample_book(TriggerCondition::AtOrBelow);
        below.insert(price(105), make_order(6, 1)).unwrap();

        let (threshold, order) = below.best_order().unwrap();
        assert_eq!((threshold.get(), order.id().get()), (105, 2));

        assert!(make_book(TriggerCondition::AtOrAbove).best_order().is_none());
        assert!(make_book(TriggerCondition::AtOrBelow).best_order().is_none());
    }

    #[test]
    fn best_order_at() {
        let book = sample_book(TriggerCondition::AtOrAbove);

        let (threshold, order) = book.best_order_at(&price(100)).unwrap();
        assert_eq!((threshold.get(), order.id().get()), (100, 1));

        let (threshold, order) = book.best_order_at(&price(105)).unwrap();
        assert_eq!((threshold.get(), order.id().get()), (105, 2));

        assert!(book.best_order_at(&price(99)).is_none());
    }

    #[test]
    fn orders_at() {
        let book = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(ref_entries(book.orders_at(&price(100))), vec![(100, 1), (100, 3)]);
        assert_eq!(ref_entries(book.orders_at(&price(105))), vec![(105, 2)]);
        assert!(book.orders_at(&price(99)).next().is_none());
    }

    #[test]
    fn orders_through() {
        let above = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(
            ref_entries(above.orders_through(Some(&price(100)))),
            vec![(95, 4), (100, 1), (100, 3)]
        );
        assert!(above.orders_through(Some(&price(94))).next().is_none());
        assert_eq!(
            ref_entries(above.orders_through(None)),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );

        let below = sample_book(TriggerCondition::AtOrBelow);

        assert_eq!(
            ref_entries(below.orders_through(Some(&price(100)))),
            vec![(100, 1), (100, 3), (105, 2)]
        );
        assert!(below.orders_through(Some(&price(110))).next().is_none());
        assert_eq!(
            ref_entries(below.orders_through(Some(&price(94)))),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );
        assert_eq!(
            ref_entries(below.orders_through(None)),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );
    }

    #[test]
    fn orders_through_quantity() {
        let above = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(
            ref_entries(above.orders_through_quantity(None)),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );
        assert_eq!(
            ref_entries(above.orders_through_quantity(Some(10.into()))),
            vec![(95, 4)]
        );
        assert_eq!(
            ref_entries(above.orders_through_quantity(Some(12.into()))),
            vec![(95, 4), (100, 1)]
        );
        assert_eq!(
            ref_entries(above.orders_through_quantity(Some(22.into()))),
            vec![(95, 4), (100, 1), (100, 3)]
        );
        assert_eq!(
            ref_entries(above.orders_through_quantity(Some(100.into()))),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );

        let below = sample_book(TriggerCondition::AtOrBelow);

        assert_eq!(
            ref_entries(below.orders_through_quantity(None)),
            vec![(105, 2), (100, 1), (100, 3), (95, 4)]
        );
        assert_eq!(
            ref_entries(below.orders_through_quantity(Some(8.into()))),
            vec![(105, 2)]
        );
        assert_eq!(
            ref_entries(below.orders_through_quantity(Some(11.into()))),
            vec![(105, 2), (100, 1)]
        );
        assert_eq!(
            ref_entries(below.orders_through_quantity(Some(20.into()))),
            vec![(105, 2), (100, 1), (100, 3)]
        );
        assert_eq!(
            ref_entries(below.orders_through_quantity(Some(21.into()))),
            vec![(105, 2), (100, 1), (100, 3), (95, 4)]
        );
    }

    #[test]
    fn quantity_at() {
        let book = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(book.quantity_at(&price(95)), 10.into());
        assert_eq!(book.quantity_at(&price(100)), 12.into());
        assert_eq!(book.quantity_at(&price(105)), 8.into());
        assert_eq!(book.quantity_at(&price(99)), 0.into());
        assert_eq!(
            make_book(TriggerCondition::AtOrAbove).quantity_at(&price(100)),
            0.into()
        );
    }

    #[test]
    fn quantity_through() {
        let above = sample_book(TriggerCondition::AtOrAbove);

        assert_eq!(above.quantity_through(Some(&price(100))), 22.into());
        assert_eq!(above.quantity_through(Some(&price(94))), 0.into());
        assert_eq!(above.quantity_through(Some(&price(110))), 30.into());
        assert_eq!(above.quantity_through(None), 30.into());

        let below = sample_book(TriggerCondition::AtOrBelow);

        assert_eq!(below.quantity_through(Some(&price(100))), 20.into());
        assert_eq!(below.quantity_through(Some(&price(110))), 0.into());
        assert_eq!(below.quantity_through(Some(&price(94))), 30.into());
        assert_eq!(below.quantity_through(None), 30.into());

        assert_eq!(make_book(TriggerCondition::AtOrBelow).quantity_through(None), 0.into());
    }
}
