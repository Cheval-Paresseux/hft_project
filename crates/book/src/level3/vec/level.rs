use crate::level3::common::OrderBookError;

use types::order::{
    fields::{OrderId, Price, Quantity},
    orders::{IncomingOrder, Order},
};

// ── Quantity Helpers ──────────────────────────────────────────────────────────

fn add_quantities(lhs: Quantity, rhs: Quantity) -> Quantity {
    (lhs + rhs).unwrap_or(Quantity::new(u64::MAX))
}

fn sub_quantities(lhs: Quantity, rhs: Quantity) -> Quantity {
    (lhs - rhs).unwrap_or(Quantity::new(0))
}

// ── Book Level ────────────────────────────────────────────────────────────────

pub struct VecL3BookLevel<R> {
    pub price: Price,
    pub total_quantity: Quantity,

    orders: Vec<R>,
}

impl<R> VecL3BookLevel<R> {
    pub fn new(price: Price) -> Self {
        Self {
            price,
            total_quantity: Quantity::new(0),
            orders: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    pub fn orders(&self) -> &[R] {
        &self.orders
    }
}

// ── Book Mutations ────────────────────────────────────────────────────────────

impl<R: Order> VecL3BookLevel<R> {
    fn find_position(&self, order_id: OrderId) -> Result<usize, OrderBookError> {
        self.orders
            .iter()
            .position(|order| order.id() == order_id)
            .ok_or(OrderBookError::OrderIdNotFound { order_id })
    }

    pub fn insert<I>(&mut self, order: I) -> Result<(), OrderBookError>
    where
        I: IncomingOrder,
        R: From<I>,
    {
        self.total_quantity = add_quantities(self.total_quantity, order.quantity());
        self.orders.push(order.into());

        Ok(())
    }

    pub fn remove(&mut self, order_id: OrderId) -> Result<(), OrderBookError> {
        let position = self.find_position(order_id)?;
        let quantity = self.orders[position].quantity();

        self.total_quantity = sub_quantities(self.total_quantity, quantity);
        self.orders.remove(position);

        Ok(())
    }

    pub fn reduce(&mut self, order_id: OrderId, remaining_quantity: Quantity) -> Result<(), OrderBookError> {
        let position = self.find_position(order_id)?;
        let old_quantity = self.orders[position].quantity();
        let delta = Quantity::new(remaining_quantity.get().abs_diff(old_quantity.get()));

        self.total_quantity = if remaining_quantity >= old_quantity {
            add_quantities(self.total_quantity, delta)
        } else {
            sub_quantities(self.total_quantity, delta)
        };

        self.orders[position].set_quantity(remaining_quantity);

        Ok(())
    }
}

// ── Peek Orders ───────────────────────────────────────────────────────────────

impl<R: Order> VecL3BookLevel<R> {
    pub fn order(&self, order_id: OrderId) -> Option<&R> {
        let position = self.find_position(order_id).ok()?;

        self.orders.get(position)
    }

    pub fn best_order(&self) -> Option<&R> {
        self.orders.first()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use types::order::{
        TimeInForce,
        fields::OrderSide,
        orders::{LimitOrder, RestingLimitOrder},
    };

    fn price(value: u64) -> Price {
        Price::new(value)
    }

    fn quantity(value: u64) -> Quantity {
        Quantity::new(value)
    }

    fn limit(id: u64, quantity: u64) -> LimitOrder {
        LimitOrder::new(
            id.into(),
            quantity.into(),
            0.into(),
            OrderSide::Ask,
            price(100),
            TimeInForce::GoodTillCanceled,
        )
    }

    fn make_level() -> VecL3BookLevel<RestingLimitOrder> {
        VecL3BookLevel::new(price(100))
    }

    #[test]
    fn new() {
        let level = make_level();

        assert_eq!(level.price, price(100));
        assert_eq!(level.total_quantity, quantity(0));
        assert!(level.orders.is_empty());
    }

    #[test]
    fn is_empty() {
        let mut level = make_level();

        assert!(level.is_empty());

        level.insert(limit(1, 10)).unwrap();
        assert!(!level.is_empty());

        level.remove(1.into()).unwrap();
        assert!(level.is_empty());
    }

    #[test]
    fn insert() {
        let mut level = make_level();

        assert_eq!(level.insert(limit(1, 10)), Ok(()));
        assert_eq!(level.orders.len(), 1);
        assert_eq!(level.orders[0].id(), 1.into());
        assert_eq!(level.orders[0].quantity(), quantity(10));
        assert_eq!(level.total_quantity, quantity(10));
    }

    #[test]
    fn remove() {
        let mut level = make_level();
        level.insert(limit(1, 10)).unwrap();
        level.insert(limit(2, 5)).unwrap();

        assert_eq!(level.remove(1.into()), Ok(()));
        assert_eq!(level.total_quantity, quantity(5));
        assert_eq!(level.orders.len(), 1);

        assert_eq!(
            level.remove(42.into()),
            Err(OrderBookError::OrderIdNotFound { order_id: 42.into() })
        );
    }

    #[test]
    fn reduce() {
        let mut level = make_level();
        level.insert(limit(1, 10)).unwrap();

        assert_eq!(level.reduce(1.into(), quantity(6)), Ok(()));
        assert_eq!(level.orders[0].quantity(), quantity(6));
        assert_eq!(level.total_quantity, quantity(6));

        assert_eq!(level.reduce(1.into(), quantity(8)), Ok(()));
        assert_eq!(level.orders[0].quantity(), quantity(8));
        assert_eq!(level.total_quantity, quantity(8));

        assert_eq!(
            level.reduce(42.into(), quantity(1)),
            Err(OrderBookError::OrderIdNotFound { order_id: 42.into() })
        );
    }

    #[test]
    fn order() {
        let mut level = make_level();
        level.insert(limit(1, 10)).unwrap();

        assert_eq!(level.order(1.into()).map(|order| order.quantity()), Some(quantity(10)));
        assert_eq!(level.order(42.into()), None);
    }

    #[test]
    fn best_order() {
        let mut level = make_level();
        level.insert(limit(1, 10)).unwrap();
        level.insert(limit(2, 5)).unwrap();

        assert_eq!(
            level.best_order().map(|order| (order.id(), order.quantity())),
            Some((1.into(), quantity(10)))
        );
        assert_eq!(make_level().best_order(), None);
    }

    #[test]
    fn orders() {
        let mut level = make_level();
        level.insert(limit(1, 10)).unwrap();
        level.insert(limit(2, 5)).unwrap();

        let orders: Vec<_> = level
            .orders()
            .iter()
            .map(|order| (order.id(), order.quantity()))
            .collect();

        assert_eq!(orders, vec![(1.into(), quantity(10)), (2.into(), quantity(5))]);
        assert!(make_level().orders().is_empty());
    }
}
