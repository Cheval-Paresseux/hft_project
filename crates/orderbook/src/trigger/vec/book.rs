use crate::trigger::common::{Direction, TriggerBook, TriggerBookError};

use types::order::{
    fields::{OrderId, Quantity},
    orders::Order,
};

// ── Vec Trigger Book ──────────────────────────────────────────────────────────

pub struct VecTriggerBook<K, V> {
    orders: Vec<(K, V)>,
    direction: Direction,
}

impl<K, V> VecTriggerBook<K, V> {
    pub fn new(direction: Direction) -> Self {
        Self {
            orders: Vec::new(),
            direction,
        }
    }
}

impl<K: Ord, V: Order> VecTriggerBook<K, V> {
    /// Number of leading orders whose trigger has been reached at `trigger`.
    ///
    /// This is also the insertion position used by `add_order`: it points past
    /// any orders with an equal trigger, preserving FIFO order.
    fn split_point(&self, trigger: &K) -> usize {
        self.orders
            .partition_point(|(t, _)| self.direction.is_triggered(t, trigger))
    }

    fn find_order_pos(&self, order_id: OrderId) -> Option<usize> {
        self.orders
            .iter()
            .position(|(_, order)| order.id() == order_id)
    }
}

// ── Trigger Book Trait ────────────────────────────────────────────────────────

impl<K: Ord, V: Order> TriggerBook<K, V> for VecTriggerBook<K, V> {
    // ── Book Mutations ────────────────────────────────────────────────────────

    fn add_order(&mut self, trigger: K, order: V) -> Result<(), TriggerBookError> {
        if self.find_order_pos(order.id()).is_some() {
            return Err(TriggerBookError::OrderIdAlreadyExists {
                order_id: order.id(),
            });
        }

        let position = self.split_point(&trigger);
        self.orders.insert(position, (trigger, order));

        Ok(())
    }

    fn cancel_order(&mut self, order_id: OrderId) -> Result<(), TriggerBookError> {
        let position = self
            .find_order_pos(order_id)
            .ok_or(TriggerBookError::OrderIdNotFound { order_id })?;

        self.orders.remove(position);

        Ok(())
    }

    fn modify_order(
        &mut self,
        order_id: OrderId,
        new_quantity: Quantity,
    ) -> Result<(), TriggerBookError> {
        let (_, order) = self
            .orders
            .iter_mut()
            .find(|(_, order)| order.id() == order_id)
            .ok_or(TriggerBookError::OrderIdNotFound { order_id })?;

        order.set_quantity(new_quantity);

        Ok(())
    }

    // ── Orders Look Up ────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(&K, &V)> {
        let position = self.find_order_pos(order_id)?;
        let (trigger, order) = &self.orders[position];

        Some((trigger, order))
    }

    fn top_order(&self) -> Option<(&K, &V)> {
        let (trigger, order) = self.orders.first()?;

        Some((trigger, order))
    }

    fn top_trigger(&self) -> Option<&K> {
        Some(&self.orders.first()?.0)
    }

    fn len(&self) -> usize {
        self.orders.len()
    }

    fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    // ── Trigger Look Up ───────────────────────────────────────────────────────

    fn direction(&self) -> Direction {
        self.direction
    }

    fn is_triggered(&self, price: &K) -> bool {
        self.split_point(price) != 0
    }

    fn orders_triggered(&self, price: &K) -> Option<Vec<(&K, &V)>> {
        let end = self.split_point(price);

        if end == 0 {
            return None;
        }

        Some(
            self.orders[..end]
                .iter()
                .map(|(trigger, order)| (trigger, order))
                .collect(),
        )
    }

    fn pop_triggered(&mut self, price: &K) -> Vec<(K, V)> {
        let end = self.split_point(price);

        self.orders.drain(..end).collect()
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

    fn ids(orders: &[(&Price, &RestingLimitOrder)]) -> Vec<OrderId> {
        orders.iter().map(|(_, order)| order.id()).collect()
    }

    #[test]
    fn add_order_keeps_increasing_triggers_sorted() {
        let mut book = VecTriggerBook::new(Direction::Increasing);

        book.add_order(price(105), make_order(1, 10)).unwrap();
        book.add_order(price(100), make_order(2, 5)).unwrap();
        book.add_order(price(110), make_order(3, 7)).unwrap();

        let triggers: Vec<_> = book.orders.iter().map(|(trigger, _)| *trigger).collect();
        assert_eq!(triggers, vec![price(100), price(105), price(110)]);
        assert_eq!(book.top_trigger(), Some(&price(100)));
    }

    #[test]
    fn add_order_keeps_decreasing_triggers_sorted() {
        let mut book = VecTriggerBook::new(Direction::Decreasing);

        book.add_order(price(100), make_order(1, 10)).unwrap();
        book.add_order(price(105), make_order(2, 5)).unwrap();
        book.add_order(price(95), make_order(3, 7)).unwrap();

        let triggers: Vec<_> = book.orders.iter().map(|(trigger, _)| *trigger).collect();
        assert_eq!(triggers, vec![price(105), price(100), price(95)]);
        assert_eq!(book.top_trigger(), Some(&price(105)));
    }

    #[test]
    fn equal_triggers_preserve_fifo_order() {
        let mut book = VecTriggerBook::new(Direction::Increasing);

        book.add_order(price(100), make_order(1, 10)).unwrap();
        book.add_order(price(100), make_order(2, 5)).unwrap();
        book.add_order(price(100), make_order(3, 7)).unwrap();

        let triggered = book.orders_triggered(&price(100)).unwrap();
        assert_eq!(ids(&triggered), vec![1.into(), 2.into(), 3.into()]);
    }

    #[test]
    fn duplicate_order_id_errors() {
        let mut book = VecTriggerBook::new(Direction::Increasing);
        book.add_order(price(100), make_order(1, 10)).unwrap();

        assert_eq!(
            book.add_order(price(105), make_order(1, 5)),
            Err(TriggerBookError::OrderIdAlreadyExists { order_id: 1.into() })
        );

        assert_eq!(book.len(), 1);
        assert_eq!(book.top_trigger(), Some(&price(100)));
    }

    #[test]
    fn cancel_order() {
        let mut book = VecTriggerBook::new(Direction::Increasing);
        book.add_order(price(100), make_order(1, 10)).unwrap();
        book.add_order(price(105), make_order(2, 5)).unwrap();

        assert_eq!(book.cancel_order(1.into()), Ok(()));
        assert_eq!(book.len(), 1);
        assert_eq!(book.top_trigger(), Some(&price(105)));

        assert_eq!(
            book.cancel_order(42.into()),
            Err(TriggerBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
    }

    #[test]
    fn modify_order() {
        let mut book = VecTriggerBook::new(Direction::Increasing);
        book.add_order(price(100), make_order(1, 10)).unwrap();

        assert_eq!(book.modify_order(1.into(), 4.into()), Ok(()));

        let (_, order) = book.order(1.into()).unwrap();
        assert_eq!(order.quantity(), 4.into());
        assert_eq!(
            book.modify_order(42.into(), 4.into()),
            Err(TriggerBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
    }

    #[test]
    fn increasing_book_triggers_on_rising_price() {
        let mut book = VecTriggerBook::new(Direction::Increasing);
        book.add_order(price(100), make_order(1, 10)).unwrap();
        book.add_order(price(105), make_order(2, 5)).unwrap();
        book.add_order(price(110), make_order(3, 7)).unwrap();

        assert!(!book.is_triggered(&price(99)));
        assert!(book.is_triggered(&price(100)));
        assert_eq!(book.orders_triggered(&price(99)), None);
        assert_eq!(
            ids(&book.orders_triggered(&price(105)).unwrap()),
            vec![1.into(), 2.into()]
        );

        let triggered = book.pop_triggered(&price(105));
        let triggered_ids: Vec<_> = triggered.iter().map(|(_, order)| order.id()).collect();
        assert_eq!(triggered_ids, vec![1.into(), 2.into()]);
        assert_eq!(book.top_trigger(), Some(&price(110)));
        assert!(book.pop_triggered(&price(105)).is_empty());
    }

    #[test]
    fn decreasing_book_triggers_on_falling_price() {
        let mut book = VecTriggerBook::new(Direction::Decreasing);
        book.add_order(price(110), make_order(1, 10)).unwrap();
        book.add_order(price(105), make_order(2, 5)).unwrap();
        book.add_order(price(100), make_order(3, 7)).unwrap();

        assert!(!book.is_triggered(&price(111)));
        assert!(book.is_triggered(&price(110)));
        assert_eq!(book.orders_triggered(&price(111)), None);
        assert_eq!(
            ids(&book.orders_triggered(&price(105)).unwrap()),
            vec![1.into(), 2.into()]
        );

        let triggered = book.pop_triggered(&price(105));
        let triggered_ids: Vec<_> = triggered.iter().map(|(_, order)| order.id()).collect();
        assert_eq!(triggered_ids, vec![1.into(), 2.into()]);
        assert_eq!(book.top_trigger(), Some(&price(100)));
        assert!(book.pop_triggered(&price(111)).is_empty());
    }

    #[test]
    fn empty_book_queries() {
        let book = VecTriggerBook::<Price, RestingLimitOrder>::new(Direction::Increasing);

        assert!(book.is_empty());
        assert_eq!(book.len(), 0);
        assert_eq!(book.top_order(), None);
        assert_eq!(book.top_trigger(), None);
        assert_eq!(book.order(1.into()), None);
        assert!(!book.is_triggered(&price(100)));
        assert_eq!(book.orders_triggered(&price(100)), None);
    }
}
