use super::config::SideConfig;
use super::level::BookLevel;
use crate::errors::{OrderBookError, SideError};
use domain::order::{LimitOrder, Modification, OrderId, OrderSide, Price, Quantity};
use std::fmt;

// ── Book Side ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct BookSide {
    pub side: OrderSide,
    config: SideConfig,
    resting_levels: Vec<BookLevel>,
}

impl BookSide {
    pub fn new(side: OrderSide, config: SideConfig, capacity: usize) -> Self {
        Self {
            side,
            config,
            resting_levels: Vec::with_capacity(capacity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.resting_levels.is_empty()
    }

    pub fn best_price(&self) -> Option<Price> {
        self.resting_levels.first().map(|level| level.price)
    }

    pub fn best_quantity(&self) -> Option<Quantity> {
        self.resting_levels.first().map(|level| level.total_quantity)
    }

    pub fn quantity_at(&self, price: Price) -> Option<Quantity> {
        if self.is_empty() {return Option::None;}

        let pos = self.find_level_pos(price);

        if pos == self.resting_levels.len() || self.resting_levels[pos].price != price {
            return Option::None;
        }

        Some(self.resting_levels[pos].total_quantity)
    }
}

impl fmt::Display for BookSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "------- Side: {} -------", self.side)?;
        
        for level in &self.resting_levels {
            writeln!(f, "{}", level)?;
        }

        Ok(())
    }
}

// ── Levels Management ─────────────────────────────────────────────────────────

impl BookSide {
    fn find_level_pos(&self, price: Price) -> usize {
        match self.side {
            OrderSide::Ask => self.resting_levels.partition_point(|l| l.price < price),
            OrderSide::Bid => self.resting_levels.partition_point(|l| l.price > price),
        }
    }

    fn get_or_create_level(&mut self, price: Price) -> &mut BookLevel {
        let pos = self.find_level_pos(price);

        if pos == self.resting_levels.len() || self.resting_levels[pos].price != price {
            self.resting_levels
                .insert(pos, BookLevel::new(price, self.config.level_capacity));
        }

        &mut self.resting_levels[pos]
    }
}

// ── Orders Management ─────────────────────────────────────────────────────────

impl BookSide {
    pub fn add_order(&mut self, order: LimitOrder) {
        self.get_or_create_level(order.price).add_order(order);
    }

    pub fn cancel_order(
        &mut self,
        order_id: OrderId,
        price_level: Price,
    ) -> Result<(), OrderBookError> {
        let pos = self.find_level_pos(price_level);

        if pos == self.resting_levels.len() || self.resting_levels[pos].price != price_level {
            return Err(SideError::CancelAtEmptyLevel { order_id, price_level, side: self.side }.into());
        }

        let empty = {
            let level = &mut self.resting_levels[pos];
            level.cancel_order(order_id)?;
            level.is_empty()
        };

        if empty {
            self.resting_levels.remove(pos);
        }

        Ok(())
    }

    pub fn modify_order(
        &mut self,
        modification: Modification,
        price_level: Price,
    ) -> Result<(), OrderBookError> {
        let pos = self.find_level_pos(price_level);

        if pos == self.resting_levels.len() || self.resting_levels[pos].price != price_level {
            return Err(SideError::ModifyAtEmptyLevel { modification_id: modification.id, price_level, side: self.side }.into());
        }

        let level = &mut self.resting_levels[pos];
        level.modify_order(modification)?;

        Ok(())
    }

    pub fn fill(
        &mut self,
        fill_quantity: Quantity,
        price_level: Price,
    ) -> Result<Vec<OrderId>, OrderBookError> {
        let pos = self.find_level_pos(price_level);

        if pos == self.resting_levels.len() || self.resting_levels[pos].price != price_level {
            return Err(SideError::FillAtEmptyLevel { fill_quantity, price_level, side: self.side }.into());
        }

        let level = &mut self.resting_levels[pos];
        let filled_orders = level.fill(fill_quantity)?;

        if self.resting_levels[pos].is_empty() {
            self.resting_levels.remove(pos);
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
    use domain::order::{ModificationId, OrderSide, Timestamp};

    fn make_bid_order(quantity: Quantity, price: Price, increment: u64) -> LimitOrder {
        let id = OrderId::new(increment);
        let timestamp = Timestamp::new(1000 + increment);
        let side = OrderSide::Bid;

        LimitOrder::new(id, timestamp, quantity, side, price)
    }

    fn make_ask_order(quantity: Quantity, price: Price, increment: u64) -> LimitOrder {
        let id = OrderId::new(increment);
        let timestamp = Timestamp::new(1000 + increment);
        let side = OrderSide::Ask;

        LimitOrder::new(id, timestamp, quantity, side, price)
    }

    fn make_bid_side() -> BookSide {
        BookSide::new(OrderSide::Bid, SideConfig::new(10),  10)
    }

    fn make_ask_side() -> BookSide {
        BookSide::new(OrderSide::Ask, SideConfig::new(10), 10)
    }

    #[test]
    fn inspect_side_size() {
        println!("Side: {} bytes", std::mem::size_of::<BookSide>());
    }

    #[test]
    fn new_side_is_empty() {
        let side = make_bid_side();

        assert!(side.is_empty());
        assert_eq!(side.resting_levels.len(), 0);
    }

    #[test]
    fn add_order_creates_new_level() {
        let mut side = make_bid_side();

        let order = make_bid_order(Quantity::new(10), Price::new(100), 0);
        side.add_order(order);

        assert_eq!(side.resting_levels.len(), 1);
        assert_eq!(side.resting_levels[0].price, Price::new(100));
    }

    #[test]
    fn add_two_orders_same_price_creates_one_level() {
        let mut side = make_bid_side();

        side.add_order(make_bid_order(Quantity::new(10), Price::new(100), 0));
        side.add_order(make_bid_order(Quantity::new(10), Price::new(100), 1));

        assert_eq!(side.resting_levels.len(), 1);
        assert_eq!(side.resting_levels[0].total_quantity, Quantity::new(20));
    }

    #[test]
    fn bid_levels_are_sorted_descending() {
        let mut side = make_bid_side();

        side.add_order(make_bid_order(Quantity::new(10), Price::new(100), 0));
        side.add_order(make_bid_order(Quantity::new(10), Price::new(105), 1));
        side.add_order(make_bid_order(Quantity::new(10), Price::new(95), 2));

        let prices: Vec<_> = side
            .resting_levels
            .iter()
            .map(|l| l.price)
            .collect();

        assert_eq!(prices, vec![Price::new(105), Price::new(100), Price::new(95)]);
    }

    #[test]
    fn ask_levels_are_sorted_ascending() {
        let mut side = make_ask_side();

        side.add_order(make_ask_order(Quantity::new(10), Price::new(100), 0));
        side.add_order(make_ask_order(Quantity::new(10), Price::new(105), 1));
        side.add_order(make_ask_order(Quantity::new(10), Price::new(95), 2));

        let prices: Vec<_> = side
            .resting_levels
            .iter()
            .map(|l| l.price)
            .collect();

        assert_eq!(prices, vec![Price::new(95), Price::new(100), Price::new(105)]);
    }

    #[test]
    fn cancel_order_removes_order() {
        let mut side = make_bid_side();

        side.add_order(make_bid_order(Quantity::new(10), Price::new(100), 0));

        let id = OrderId::new(0);
        let result = side.cancel_order(id, Price::new(100));

        assert!(result.is_ok());
    }

    #[test]
    fn cancel_last_order_removes_level() {
        let mut side = make_bid_side();

        side.add_order(make_bid_order(Quantity::new(10), Price::new(100), 0));

        assert_eq!(side.resting_levels.len(), 1);

        let id = OrderId::new(0);
        side.cancel_order(id, Price::new(100)).unwrap();

        assert!(side.resting_levels.is_empty());
    }

    #[test]
    fn cancel_unknown_level_returns_error() {
        let mut side = make_bid_side();

        let result = side.cancel_order(
            OrderId::new(1),
            Price::new(100),
        );

        assert!(result.is_err());
    }

    #[test]
    fn fill_unknown_level_returns_error() {
        let mut side = make_bid_side();

        let result = side.fill(Quantity::new(10), Price::new(100));

        assert!(result.is_err());
    }

    #[test]
    fn modify_unknown_level_returns_error() {
        let mut side = make_bid_side();

        let modification = Modification::new_reduce_quantity(
            ModificationId::new(1),
            Timestamp::new(1000),
            OrderId::new(999),
            Quantity::new(5),
        );

        let result = side.modify_order(
            modification,
            Price::new(100),
        );

        assert!(result.is_err());
    }
}