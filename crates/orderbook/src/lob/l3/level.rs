use crate::errors::{LevelError, OrderBookError};
use domain::order::{
    LimitOrder, LimitRestingOrder, Modification, ModificationInstruction, OrderId, Price, Quantity,
};
use std::fmt;

// ── Book Level ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct BookLevel {
    pub price: Price,
    pub total_quantity: Quantity,

    resting_orders: Vec<LimitRestingOrder>,
}

impl BookLevel {
    pub fn new(price: Price, capacity: usize) -> Self {
        Self {
            price,
            resting_orders: Vec::with_capacity(capacity),
            total_quantity: Quantity::new(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.resting_orders.is_empty()
    }

    fn limit_to_resting_order(&self, order: LimitOrder) -> LimitRestingOrder {
        LimitRestingOrder::new(order.id, order.timestamp, order.quantity)
    }
}

impl fmt::Display for BookLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "* {} - Total quantity: {}", self.price, self.total_quantity)?;
        
        for order in &self.resting_orders {
            writeln!(f, "[ Qty: {} - Id: {} ]", order.quantity, order.id)?;
        }

        Ok(())
    }
}

// ── Level Management ────────────────────────────────────────────────────────────

impl BookLevel {
    fn find_order_position(&self, order_id: OrderId) -> Result<usize, OrderBookError> {
        self.resting_orders
            .iter()
            .position(|o| o.id == order_id)
            .ok_or_else(|| {
                LevelError::IdNotFound {
                    order_id,
                    price: self.price,
                }
                .into()
            })
    }

    pub fn add_order(&mut self, order: LimitOrder) {
        debug_assert!(
            !self.resting_orders.iter().any(|o| o.id == order.id),
            "duplicate order id {} at price {}",
            order.id,
            self.price
        );

        let pos = self
            .resting_orders
            .partition_point(|o| o.timestamp <= order.timestamp);
        let o = self.limit_to_resting_order(order);
        self.resting_orders.insert(pos, o);

        self.total_quantity = self
            .total_quantity
            .add(order.quantity)
            .expect("total_quantity overflow in add_order");
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Result<(), OrderBookError> {
        let pos = self.find_order_position(order_id)?;
        let order = self.resting_orders.remove(pos);

        self.total_quantity = self
            .total_quantity
            .sub(order.quantity)
            .expect("total_quantity underflow in cancel_order");

        Ok(())
    }

    pub fn modify_order(&mut self, modification: Modification) -> Result<(), OrderBookError> {
        let pos: usize = self.find_order_position(modification.order_id)?;

        match modification.instruction {
            ModificationInstruction::ReduceQuantity(qty) => {
                self.reduce_order_quantity(pos, modification, qty)
            }
            ModificationInstruction::IncreaseQuantity(qty) => {
                self.increase_order_quantity(pos, modification, qty)
            }

            _ => Err(LevelError::ModificationNotHandled {
                modification_id: modification.id,
                order_id: modification.order_id,
                price: self.price,
            }
            .into()),
        }
    }
}

// ── Order Modification ──────────────────────────────────────────────────────────

impl BookLevel {
    fn reduce_order_quantity(
        &mut self,
        pos: usize,
        modification: Modification,
        new_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let quantity_delta = new_quantity.delta(self.resting_orders[pos].quantity);
        self.resting_orders[pos].modify(modification)?;
        self.total_quantity = self
            .total_quantity
            .sub(quantity_delta)
            .expect("total_quantity underflow in reduce_order_quantity");

        Ok(())
    }

    fn increase_order_quantity(
        &mut self,
        pos: usize,
        modification: Modification,
        new_quantity: Quantity,
    ) -> Result<(), OrderBookError> {
        let quantity_delta = new_quantity.delta(self.resting_orders[pos].quantity);
        self.resting_orders[pos].modify(modification)?;
        self.total_quantity = self
            .total_quantity
            .add(quantity_delta)
            .expect("total_quantity overflow in increase_order_quantity");

        let mut order = self.resting_orders.remove(pos);
        order.timestamp = modification.timestamp;

        let pos = self
            .resting_orders
            .partition_point(|o| o.timestamp <= order.timestamp);
        self.resting_orders.insert(pos, order);

        Ok(())
    }
}

// ── Fill Management ─────────────────────────────────────────────────────────────

impl BookLevel {
    fn fill_first_order(&mut self, fill_quantity: Quantity) -> Result<(Quantity, Option<OrderId>), OrderBookError> {
        let first_order = &self.resting_orders[0];
        let order_id = first_order.id;

        if fill_quantity < first_order.quantity {
            self.resting_orders[0].fill(fill_quantity)?;
            self.total_quantity = self
                .total_quantity
                .sub(fill_quantity)
                .expect("total_quantity underflow in fill_first_order");

            Ok((Quantity::new(0), None))
        } else {
            let filled = first_order.quantity;
            let remainder = fill_quantity
                .sub(filled)
                .expect("fill_quantity underflow in fill_first_order");

            self.resting_orders[0].fill(filled)?;
            self.total_quantity = self
                .total_quantity
                .sub(filled)
                .expect("total_quantity underflow in fill_first_order");
            self.resting_orders.remove(0);

            Ok((remainder, Some(order_id)))
        }
    }

    pub fn fill(&mut self, mut fill_quantity: Quantity) -> Result<Vec<OrderId>, OrderBookError> {
        let mut filled_orders = Vec::new();
        
        while fill_quantity > Quantity::new(0) && !self.resting_orders.is_empty() {
            let (remainder, filled_id) = self.fill_first_order(fill_quantity)?;

            if let Some(order_id) = filled_id {
                filled_orders.push(order_id);
            }

            fill_quantity = remainder;
        }

        Ok(filled_orders)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use domain::order::{
        MarketTimeInForce, ModificationId, OrderId, OrderInstruction, OrderSide, Price, Quantity,
        Timestamp,
    };

    fn make_order(quantity: Quantity, increment: u64) -> LimitOrder {
        let id = OrderId::new(increment);
        let timestamp = Timestamp::new(1000 + increment);
        let side = OrderSide::Bid;
        let price = Price::new(100);

        LimitOrder::new(id, timestamp, quantity, side, price)
    }

    #[test]
    fn inspect_level_size() {
        println!("Level: {} bytes", std::mem::size_of::<BookLevel>());
    }

    #[test]
    fn new_level() {
        let level = BookLevel::new(Price::new(100), 10);

        assert_eq!(level.total_quantity, Quantity::new(0));
    }

    #[test]
    fn fill_partial_leaves_order_in_place() {
        let mut level = BookLevel::new(Price::new(100), 10);
        let o = make_order(Quantity::new(20), 0);
        level.add_order(o);

        assert!(level.fill(Quantity::new(8)).is_ok());

        assert_eq!(level.resting_orders.len(), 1);
        assert_eq!(level.resting_orders[0].quantity, Quantity::new(12));
        assert_eq!(level.total_quantity, Quantity::new(12));
    }

    #[test]
    fn fill_exact_removes_order() {
        let mut level = BookLevel::new(Price::new(100), 10);
        let o = make_order(Quantity::new(20), 0);
        level.add_order(o);

        assert!(level.fill(Quantity::new(20)).is_ok());

        assert!(level.is_empty());
        assert_eq!(level.total_quantity, Quantity::new(0));
    }

    #[test]
    fn fill_spans_multiple_orders() {
        let mut level = BookLevel::new(Price::new(100), 10);
        let o1 = make_order(Quantity::new(10), 0);
        let o2 = make_order(Quantity::new(10), 1);
        let o3 = make_order(Quantity::new(10), 2);
        level.add_order(o1);
        level.add_order(o2);
        level.add_order(o3);

        assert!(level.fill(Quantity::new(25)).is_ok());

        assert_eq!(level.resting_orders.len(), 1);
        assert_eq!(level.resting_orders[0].id, o3.id);
        assert_eq!(level.resting_orders[0].quantity, Quantity::new(5));
        assert_eq!(level.total_quantity, Quantity::new(5));
    }

    #[test]
    fn fill_exactly_drains_level() {
        let mut level = BookLevel::new(Price::new(100), 10);
        level.add_order(make_order(Quantity::new(10), 0));
        level.add_order(make_order(Quantity::new(10), 1));
        level.add_order(make_order(Quantity::new(10), 2));

        assert!(level.fill(Quantity::new(30)).is_ok());

        assert!(level.is_empty());
        assert_eq!(level.total_quantity, Quantity::new(0));
    }

    #[test]
    fn fill_zero_quantity_is_noop() {
        let mut level = BookLevel::new(Price::new(100), 10);
        let o = make_order(Quantity::new(10), 0);
        level.add_order(o);

        assert!(level.fill(Quantity::new(0)).is_ok());

        assert_eq!(level.resting_orders.len(), 1);
        assert_eq!(level.total_quantity, Quantity::new(10));
    }

    #[test]
    fn fill_respects_time_priority() {
        let mut level = BookLevel::new(Price::new(100), 10);
        let o1 = make_order(Quantity::new(10), 0);
        let o2 = make_order(Quantity::new(10), 1);
        level.add_order(o2);
        level.add_order(o1);

        assert!(level.fill(Quantity::new(15)).is_ok());

        assert_eq!(level.resting_orders.len(), 1);
        assert_eq!(level.resting_orders[0].id, o2.id);
        assert_eq!(level.resting_orders[0].quantity, Quantity::new(5));
    }

    #[test]
    fn add_order() {
        let mut level = BookLevel::new(Price::new(100), 10);
        let order = make_order(Quantity::new(10), 0);
        level.add_order(order);

        assert_eq!(level.resting_orders.len(), 1);
        assert_eq!(level.total_quantity, Quantity::new(10));
    }

    #[test]
    fn add_order_preserves_time_priority() {
        let mut level = BookLevel::new(Price::new(100), 10);
        let late = make_order(Quantity::new(5), 2);
        let early = make_order(Quantity::new(10), 0);
        let mid = make_order(Quantity::new(7), 1);
        level.add_order(late);
        level.add_order(early);
        level.add_order(mid);

        assert_eq!(level.resting_orders[0].id, early.id);
        assert_eq!(level.resting_orders[1].id, mid.id);
        assert_eq!(level.resting_orders[2].id, late.id);
        assert_eq!(level.total_quantity, Quantity::new(22));
    }

    #[test]
    fn cancel_order_removes_and_adjusts_quantity() {
        let mut level = BookLevel::new(Price::new(200), 10);
        let o1 = make_order(Quantity::new(10), 0);
        let o2 = make_order(Quantity::new(15), 1);
        level.add_order(o1);
        level.add_order(o2);

        assert!(level.cancel_order(o1.id).is_ok());

        assert_eq!(level.resting_orders.len(), 1);
        assert_eq!(level.resting_orders[0].id, o2.id);
        assert_eq!(level.total_quantity, Quantity::new(15));
    }

    #[test]
    fn cancel_order_unknown_id_returns_err() {
        let mut level = BookLevel::new(Price::new(200), 10);
        level.add_order(make_order(Quantity::new(10), 0));

        let result = level.cancel_order(OrderId::new(999));

        assert!(result.is_err());
    }

    #[test]
    fn cancel_all_orders_leaves_level_empty() {
        let mut level = BookLevel::new(Price::new(100), 10);
        let o = make_order(Quantity::new(5), 0);
        level.add_order(o);
        level.cancel_order(o.id).unwrap();

        assert!(level.is_empty());
        assert_eq!(level.total_quantity, Quantity::new(0));
    }

    #[test]
    fn modify_reduce_quantity_adjusts_total() {
        let mut level = BookLevel::new(Price::new(100), 10);
        let o = make_order(Quantity::new(20), 0);
        level.add_order(o);

        let modification = Modification::new_reduce_quantity(
            ModificationId::new(1),
            o.timestamp,
            o.id,
            Quantity::new(12),
        );

        assert!(level.modify_order(modification).is_ok());

        assert_eq!(level.resting_orders[0].quantity, Quantity::new(12));
        assert_eq!(level.total_quantity, Quantity::new(12));
    }

    #[test]
    fn modify_increase_quantity_requeues_order() {
        let mut level = BookLevel::new(Price::new(100), 10);
        let o1 = make_order(Quantity::new(10), 0);
        let o2 = make_order(Quantity::new(5), 1);
        level.add_order(o1);
        level.add_order(o2);

        let new_ts = Timestamp::new(2000);
        let modification = Modification::new_increase_quantity(
            ModificationId::new(1),
            new_ts,
            o1.id,
            Quantity::new(20),
        );

        assert!(level.modify_order(modification).is_ok());

        assert_eq!(level.resting_orders[0].id, o2.id);
        assert_eq!(level.resting_orders[1].id, o1.id);
        assert_eq!(level.total_quantity, Quantity::new(25));
    }

    #[test]
    fn modify_unknown_id_returns_err() {
        let mut level = BookLevel::new(Price::new(100), 10);
        level.add_order(make_order(Quantity::new(10), 0));

        let modification = Modification::new_reduce_quantity(
            ModificationId::new(1),
            Timestamp::new(1000),
            OrderId::new(999),
            Quantity::new(5),
        );

        assert!(level.modify_order(modification).is_err());
    }

    #[test]
    fn modify_unsupported_instruction_returns_err() {
        let mut level = BookLevel::new(Price::new(100), 10);
        let o = make_order(Quantity::new(10), 0);
        level.add_order(o);

        let modification = Modification::new_change_instruction(
            ModificationId::new(1),
            o.timestamp,
            o.id,
            OrderInstruction::Market { tif: MarketTimeInForce::FillOrKill },
        );

        assert!(level.modify_order(modification).is_err());
    }
}
