use types::order::{fields::Quantity, orders::Order};

// ── Orders ───────────────────────────────────────────────────────────────────

pub struct VecOrders<'a, T, O> {
    orders: &'a [(T, O)],

    index: usize,

    group_start: usize,
    group_end: usize,
    reverse: bool,

    cumulative: Quantity,
    bound: Option<Quantity>,
}

impl<'a, T, O> VecOrders<'a, T, O> {
    pub fn forward(orders: &'a [(T, O)], bound: Option<Quantity>) -> Self {
        Self {
            orders,
            index: 0,
            group_start: 0,
            group_end: 0,
            reverse: false,
            cumulative: Quantity::new(0),
            bound,
        }
    }

    pub fn reverse(orders: &'a [(T, O)], bound: Option<Quantity>) -> Self {
        let len = orders.len();

        Self {
            orders,
            index: len,
            group_start: len,
            group_end: len,
            reverse: true,
            cumulative: Quantity::new(0),
            bound,
        }
    }
}

// ── Iterator Trait ───────────────────────────────────────────────────────────

impl<'a, T: Ord, O: Order> Iterator for VecOrders<'a, T, O> {
    type Item = (&'a T, &'a O);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(bound) = self.bound
            && self.cumulative >= bound
        {
            return None;
        }

        let index = if self.reverse {
            if self.index == self.group_end {
                if self.group_start == 0 {
                    return None;
                }

                self.group_end = self.group_start;

                let group_threshold = &self.orders[self.group_end - 1].0;
                self.group_start =
                    self.orders[..self.group_end].partition_point(|(threshold, _)| threshold < group_threshold);

                self.index = self.group_start;
            }

            self.index
        } else {
            if self.index >= self.orders.len() {
                return None;
            }

            self.index
        };

        let (threshold, order) = &self.orders[index];

        if self.bound.is_some() {
            self.cumulative = (self.cumulative + order.quantity())?;
        }

        self.index += 1;

        Some((threshold, order))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use types::order::{fields::Price, orders::RestingLimitOrder};

    fn price(value: u64) -> Price {
        Price::new(value)
    }

    fn order(id: u64, quantity: u64) -> RestingLimitOrder {
        RestingLimitOrder::new(id.into(), quantity.into(), 0.into())
    }

    fn sample() -> Vec<(Price, RestingLimitOrder)> {
        vec![
            (price(95), order(4, 10)),
            (price(100), order(1, 5)),
            (price(100), order(3, 7)),
            (price(105), order(2, 8)),
        ]
    }

    fn entries<'a>(orders: impl IntoIterator<Item = (&'a Price, &'a RestingLimitOrder)>) -> Vec<(u64, u64)> {
        orders
            .into_iter()
            .map(|(threshold, order)| (threshold.get(), order.id().get()))
            .collect()
    }

    #[test]
    fn forward() {
        let orders = sample();
        let empty: Vec<(Price, RestingLimitOrder)> = Vec::new();

        assert_eq!(
            entries(VecOrders::forward(&orders, None)),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );
        assert_eq!(
            entries(VecOrders::forward(&orders[1..3], None)),
            vec![(100, 1), (100, 3)]
        );
        assert!(VecOrders::forward(&empty, None).next().is_none());

        assert!(entries(VecOrders::forward(&orders, Some(Quantity::new(0)))).is_empty());
        assert_eq!(
            entries(VecOrders::forward(&orders, Some(Quantity::new(10)))),
            vec![(95, 4)]
        );
        assert_eq!(
            entries(VecOrders::forward(&orders, Some(Quantity::new(12)))),
            vec![(95, 4), (100, 1)]
        );
        assert_eq!(
            entries(VecOrders::forward(&orders, Some(Quantity::new(21)))),
            vec![(95, 4), (100, 1), (100, 3)]
        );
        assert_eq!(
            entries(VecOrders::forward(&orders, Some(Quantity::new(22)))),
            vec![(95, 4), (100, 1), (100, 3)]
        );
        assert_eq!(
            entries(VecOrders::forward(&orders, Some(Quantity::new(30)))),
            vec![(95, 4), (100, 1), (100, 3), (105, 2)]
        );

        let overflowing = vec![
            (
                price(95),
                RestingLimitOrder::new(1.into(), Quantity::new(u64::MAX - 5), 0.into()),
            ),
            (price(100), order(2, 10)),
        ];
        assert_eq!(
            entries(VecOrders::forward(&overflowing, Some(Quantity::new(u64::MAX)))),
            vec![(95, 1)]
        );
    }

    #[test]
    fn reverse() {
        let orders = sample();
        let empty: Vec<(Price, RestingLimitOrder)> = Vec::new();

        assert_eq!(
            entries(VecOrders::reverse(&orders, None)),
            vec![(105, 2), (100, 1), (100, 3), (95, 4)]
        );
        assert!(VecOrders::reverse(&empty, None).next().is_none());

        assert!(entries(VecOrders::reverse(&orders, Some(Quantity::new(0)))).is_empty());
        assert_eq!(
            entries(VecOrders::reverse(&orders, Some(Quantity::new(8)))),
            vec![(105, 2)]
        );
        assert_eq!(
            entries(VecOrders::reverse(&orders, Some(Quantity::new(11)))),
            vec![(105, 2), (100, 1)]
        );
        assert_eq!(
            entries(VecOrders::reverse(&orders, Some(Quantity::new(20)))),
            vec![(105, 2), (100, 1), (100, 3)]
        );
        assert_eq!(
            entries(VecOrders::reverse(&orders, Some(Quantity::new(21)))),
            vec![(105, 2), (100, 1), (100, 3), (95, 4)]
        );
        assert_eq!(
            entries(VecOrders::reverse(&orders, Some(Quantity::new(30)))),
            vec![(105, 2), (100, 1), (100, 3), (95, 4)]
        );

        let fifo = vec![
            (price(100), order(1, 1)),
            (price(100), order(2, 1)),
            (price(100), order(3, 1)),
        ];
        assert_eq!(
            entries(VecOrders::reverse(&fifo, None)),
            vec![(100, 1), (100, 2), (100, 3)]
        );

        let overflowing = vec![
            (price(95), order(2, 10)),
            (
                price(100),
                RestingLimitOrder::new(1.into(), Quantity::new(u64::MAX - 5), 0.into()),
            ),
        ];
        assert_eq!(
            entries(VecOrders::reverse(&overflowing, Some(Quantity::new(u64::MAX)))),
            vec![(100, 1)]
        );
    }
}
