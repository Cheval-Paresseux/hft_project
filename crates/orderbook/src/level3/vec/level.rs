use crate::level3::common::{add_quantities, sub_quantities, OrderBookError};

use types::order::{
    fields::{OrderId, Price, Quantity},
    orders::{LimitOrder, Order, RestingLimitOrder},
};

// ── Book Level ────────────────────────────────────────────────────────────────

pub struct VecL3BookLevel {
    pub price: Price,
    pub total_quantity: Quantity,
    orders: Vec<RestingLimitOrder>,
}

impl VecL3BookLevel {
    pub fn new(price: Price) -> Self {
        Self {
            price,
            total_quantity: 0.into(),
            orders: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    fn find_position(&self, order_id: OrderId) -> Result<usize, OrderBookError> {
        self.orders
            .iter()
            .position(|o| o.id() == order_id)
            .ok_or(OrderBookError::OrderIdNotFound { order_id })
    }
}

// ── Book Mutations ────────────────────────────────────────────────────────────

impl VecL3BookLevel {
    pub fn add_order(&mut self, order: LimitOrder) -> Result<(), OrderBookError> {
        self.total_quantity = add_quantities(self.total_quantity, order.quantity())?;
        self.orders.push(order.into());

        Ok(())
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Result<(), OrderBookError> {
        let position = self.find_position(order_id)?;
        self.total_quantity =
            sub_quantities(self.total_quantity, self.orders[position].quantity())?;
        self.orders.remove(position);

        Ok(())
    }

    pub fn modify_order(
        &mut self,
        order_id: OrderId,
        new_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let position = self.find_position(order_id)?;
        let old_quantity = self.orders[position].quantity();
        let delta = Quantity::new(new_quantity.get().abs_diff(old_quantity.get()));

        if new_quantity >= old_quantity {
            self.total_quantity = add_quantities(self.total_quantity, delta)?;
        } else {
            self.total_quantity = sub_quantities(self.total_quantity, delta)?;
        }

        self.orders[position].set_quantity(new_quantity);

        Ok(())
    }

    pub fn fill_order(
        &mut self,
        order_id: OrderId,
        fill_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let position = self.find_position(order_id)?;
        self.orders[position].fill(fill_quantity)?;
        self.total_quantity = sub_quantities(self.total_quantity, fill_quantity)?;

        if self.orders[position].quantity().is_zero() {
            self.orders.remove(position);
        }

        Ok(())
    }
}

// ── Orders Look Up  ───────────────────────────────────────────────────────────

impl VecL3BookLevel {
    pub fn order(&self, order_id: OrderId) -> Option<Quantity> {
        let position = self.find_position(order_id).ok()?;

        Some(self.orders[position].quantity())
    }

    pub fn top_order(&self) -> Option<(OrderId, Quantity)> {
        let first = self.orders.first()?;

        Some((first.id(), first.quantity()))
    }

    pub fn orders_at(&self) -> Option<Vec<(OrderId, Quantity)>> {
        let mut orders = Vec::with_capacity(self.orders.len());
        for order in &self.orders {
            orders.push((order.id(), order.quantity()));
        }

        (!orders.is_empty()).then_some(orders)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use types::order::{orders::OrderError, OrderSide, TimeInForce};

    fn make_limit(id: u64, quantity: u64) -> LimitOrder {
        LimitOrder::new(
            id.into(),
            quantity.into(),
            0.into(),
            OrderSide::Ask,
            100.into(),
            TimeInForce::GoodTillCanceled,
        )
    }

    #[test]
    fn unknown_order_errors() {
        let mut level = VecL3BookLevel::new(100.into());

        assert_eq!(
            level.cancel_order(42.into()),
            Err(OrderBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );

        assert_eq!(
            level.modify_order(42.into(), 5.into()),
            Err(OrderBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );

        assert_eq!(
            level.fill_order(42.into(), 5.into()),
            Err(OrderBookError::OrderIdNotFound {
                order_id: 42.into()
            })
        );
    }

    #[test]
    fn add_order() {
        let mut level = VecL3BookLevel::new(100.into());

        assert_eq!(level.add_order(make_limit(1, 10)), Ok(()));
        assert_eq!(level.orders.len(), 1);
        assert_eq!(level.orders[0].id(), 1.into());
        assert_eq!(level.orders[0].quantity(), 10.into());
        assert_eq!(level.total_quantity, 10.into());
    }

    #[test]
    fn add_order_overflow_leaves_level_unchanged() {
        let mut level = VecL3BookLevel::new(100.into());
        level.add_order(make_limit(1, u64::MAX)).unwrap();

        assert_eq!(
            level.add_order(make_limit(2, 1)),
            Err(OrderBookError::QuantityOverflow)
        );

        assert_eq!(level.orders.len(), 1);
        assert_eq!(level.total_quantity, u64::MAX.into());
    }

    #[test]
    fn cancel_order() {
        let mut level = VecL3BookLevel::new(100.into());
        level.add_order(make_limit(1, 10)).unwrap();

        assert_eq!(level.cancel_order(1.into()), Ok(()));
        assert_eq!(level.total_quantity, 0.into());
        assert!(level.is_empty());
    }

    #[test]
    fn modify_order() {
        let mut level = VecL3BookLevel::new(100.into());
        level.add_order(make_limit(1, 10)).unwrap();

        assert_eq!(level.modify_order(1.into(), 23.into()), Ok(()));
        assert_eq!(level.orders[0].quantity(), 23.into());
        assert_eq!(level.total_quantity, 23.into());

        assert_eq!(level.modify_order(1.into(), 4.into()), Ok(()));
        assert_eq!(level.orders[0].quantity(), 4.into());
        assert_eq!(level.total_quantity, 4.into());
    }

    #[test]
    fn fill_order() {
        let mut level = VecL3BookLevel::new(100.into());
        level.add_order(make_limit(1, 10)).unwrap();

        assert_eq!(level.fill_order(1.into(), 4.into()), Ok(()));
        assert_eq!(level.orders[0].quantity(), 6.into());
        assert_eq!(level.total_quantity, 6.into());

        assert_eq!(
            level.fill_order(1.into(), 7.into()),
            Err(OrderBookError::OrderError(
                OrderError::FillExceedsOrderQuantity {
                    order_id: 1.into(),
                    order_quantity: 6.into(),
                    fill_quantity: 7.into(),
                }
            ))
        );
        assert_eq!(level.orders[0].quantity(), 6.into());
        assert_eq!(level.total_quantity, 6.into());
    }

    #[test]
    fn fully_filled_order_is_removed() {
        let mut level = VecL3BookLevel::new(100.into());
        level.add_order(make_limit(1, 10)).unwrap();

        assert_eq!(level.fill_order(1.into(), 10.into()), Ok(()));

        assert!(level.is_empty());
        assert_eq!(level.total_quantity, 0.into());
        assert_eq!(level.order(1.into()), None);
    }

    #[test]
    fn order() {
        let mut level = VecL3BookLevel::new(100.into());
        level.add_order(make_limit(1, 10)).unwrap();

        assert_eq!(level.order(1.into()), Some(10.into()));
        assert_eq!(level.order(42.into()), None);
    }

    #[test]
    fn top_order() {
        let mut level = VecL3BookLevel::new(100.into());
        level.add_order(make_limit(1, 10)).unwrap();
        level.add_order(make_limit(2, 5)).unwrap();

        assert_eq!(level.top_order(), Some((1.into(), 10.into())));
        assert_eq!(VecL3BookLevel::new(100.into()).top_order(), None);
    }

    #[test]
    fn orders_at() {
        let mut level = VecL3BookLevel::new(100.into());
        level.add_order(make_limit(1, 10)).unwrap();
        level.add_order(make_limit(2, 5)).unwrap();

        let expected_result = vec![(1.into(), 10.into()), (2.into(), 5.into())];
        assert_eq!(level.orders_at(), Some(expected_result));
        assert_eq!(VecL3BookLevel::new(100.into()).orders_at(), None);
    }
}
