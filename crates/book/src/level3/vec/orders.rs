use super::level::VecL3BookLevel;

use types::order::{
    fields::{OrderSide, Price},
    orders::Order,
};

// ── Resting Orders ───────────────────────────────────────────────────────────

/// Zero-allocation iterator over the resting orders of a book side.
///
/// The iterator walks a borrowed slice of price levels in book order and yields
/// each order together with its side and price, skipping empty levels.
pub struct VecRestingOrders<'a, R> {
    side: OrderSide,
    levels: &'a [VecL3BookLevel<R>],

    level_index: usize,
    order_index: usize,
}

impl<'a, R> VecRestingOrders<'a, R> {
    pub(crate) fn new(side: OrderSide, levels: &'a [VecL3BookLevel<R>]) -> Self {
        Self {
            side,
            levels,
            level_index: 0,
            order_index: 0,
        }
    }
}

// ── Iterator Trait ───────────────────────────────────────────────────────────

impl<'a, R: Order> Iterator for VecRestingOrders<'a, R> {
    type Item = (OrderSide, Price, &'a R);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(level) = self.levels.get(self.level_index) {
            if let Some(order) = level.orders().get(self.order_index) {
                let price = level.price;

                self.order_index += 1;

                return Some((self.side, price, order));
            }

            self.level_index += 1;
            self.order_index = 0;
        }

        None
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use types::order::{
        orders::{LimitOrder, RestingLimitOrder},
        TimeInForce,
    };

    fn price(value: u64) -> Price {
        Price::new(value)
    }

    fn limit(id: u64, quantity: u64, price: u64) -> LimitOrder {
        LimitOrder::new(
            id.into(),
            quantity.into(),
            0.into(),
            OrderSide::Ask,
            Price::new(price),
            TimeInForce::GoodTillCanceled,
        )
    }

    fn levels() -> Vec<VecL3BookLevel<RestingLimitOrder>> {
        let mut first = VecL3BookLevel::new(price(100));
        first.insert(limit(1, 5, 100)).unwrap();
        first.insert(limit(2, 7, 100)).unwrap();

        let mut second = VecL3BookLevel::new(price(105));
        second.insert(limit(3, 8, 105)).unwrap();

        vec![first, second]
    }

    fn entries<'a>(
        orders: impl IntoIterator<Item = (OrderSide, Price, &'a RestingLimitOrder)>,
    ) -> Vec<(OrderSide, Price, u64, u64)> {
        orders
            .into_iter()
            .map(|(side, price, order)| (side, price, order.id().get(), order.quantity().get()))
            .collect()
    }

    #[test]
    fn new() {
        let levels = levels();
        let orders = VecRestingOrders::new(OrderSide::Ask, &levels);

        assert_eq!(orders.side, OrderSide::Ask);
        assert_eq!(orders.levels.len(), 2);
        assert_eq!(orders.level_index, 0);
        assert_eq!(orders.order_index, 0);
    }

    #[test]
    fn next() {
        let levels = levels();

        assert_eq!(
            entries(VecRestingOrders::new(OrderSide::Ask, &levels)),
            vec![
                (OrderSide::Ask, price(100), 1, 5),
                (OrderSide::Ask, price(100), 2, 7),
                (OrderSide::Ask, price(105), 3, 8),
            ]
        );

        let empty: Vec<VecL3BookLevel<RestingLimitOrder>> = Vec::new();
        assert!(VecRestingOrders::new(OrderSide::Bid, &empty).next().is_none());
    }
}
