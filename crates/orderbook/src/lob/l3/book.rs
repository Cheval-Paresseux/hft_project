use super::config::{BookConfig, SideConfig};
use super::side::BookSide;
use crate::errors::OrderBookError;
use domain::asset::AssetId;
use domain::order::{LimitOrder, OrderId, OrderSide, Price, Modification, Quantity};
use std::fmt;
use std::collections::HashMap;

// ── OrderLookup ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TopOfBook {
    pub side: OrderSide,
    pub price: Option<Price>,
    pub quantity: Option<Quantity>,
}

// ── TopOfBook ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrderLookup {
    side: OrderSide, 
    price: Price,
}

// ── Book ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Book {
    pub asset_id: AssetId,
    book_config: BookConfig,
    bid_side: BookSide,
    ask_side: BookSide,
    order_lookup: HashMap<OrderId, OrderLookup>,
}

impl Book {
    pub fn new(
        asset_id: AssetId, 
        book_config: BookConfig, 
        bid_side_config: SideConfig, 
        ask_side_config: SideConfig
    ) -> Self {
        let lookup_capacity = book_config.lookup_capacity;
        let bid_side_capacity = book_config.bid_side_capacity;
        let ask_side_capacity = book_config.ask_side_capacity;

        Self {
            asset_id,
            book_config,
            bid_side: BookSide::new(OrderSide::Bid, bid_side_config, bid_side_capacity),
            ask_side: BookSide::new(OrderSide::Ask, ask_side_config, ask_side_capacity),
            order_lookup: HashMap::with_capacity(lookup_capacity),
        }
    }

    pub fn bid_top_of_book(&self) -> TopOfBook {
        TopOfBook {
            side: OrderSide::Bid,
            price: self.bid_side.best_price(),
            quantity: self.bid_side.best_quantity()
        }
    }

    pub fn ask_top_of_book(&self) -> TopOfBook {
        TopOfBook {
            side: OrderSide::Ask,
            price: self.ask_side.best_price(),
            quantity: self.ask_side.best_quantity()
        }
    }

    pub fn quantity_at(&self, side: OrderSide, price: Price) -> Option<Quantity> {
        match side {
            OrderSide::Bid => self.bid_side.quantity_at(price),
            OrderSide::Ask => self.ask_side.quantity_at(price),
        }
    }
}

impl fmt::Display for Book {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "========== Book: {} ==========", self.asset_id)?;

        writeln!(f, "{}", self.bid_side)?;
        writeln!(f, "{}", self.ask_side)?;

        Ok(())
    }
}

// ── Orders Management ─────────────────────────────────────────────────────────

impl Book {
    pub fn add_order(&mut self, order: LimitOrder) {
        let lookup = OrderLookup {
            side: order.side,
            price: order.price,
        };

        let order_id = order.id;

        match order.side {
            OrderSide::Ask => self.ask_side.add_order(order),
            OrderSide::Bid => self.bid_side.add_order(order),
        }

        self.order_lookup.insert(order_id, lookup);
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Result<(), OrderBookError> {
        let lookup = *self
            .order_lookup
            .get(&order_id)
            .ok_or(OrderBookError::IdNotFound { order_id })?;

        match lookup.side {
            OrderSide::Ask => self.ask_side.cancel_order(order_id, lookup.price)?,
            OrderSide::Bid => self.bid_side.cancel_order(order_id, lookup.price)?,
        };

        self.order_lookup.remove(&order_id);

        Ok(())
    }

    pub fn modify_order(&mut self, modification: Modification) -> Result<(), OrderBookError> {
        let lookup = *self
            .order_lookup
            .get(&modification.order_id)
            .ok_or(OrderBookError::IdNotFound { order_id: modification.order_id })?;

        match lookup.side {
            OrderSide::Ask => self.ask_side.modify_order(modification, lookup.price)?,
            OrderSide::Bid => self.bid_side.modify_order(modification, lookup.price)?,
        };

        Ok(())
    }

    pub fn fill(
        &mut self,
        fill_quantity: Quantity,
        price_level: Price,
        side: OrderSide,
    ) -> Result<(), OrderBookError> {
        let filled_orders = match side {
            OrderSide::Ask => self.ask_side.fill(fill_quantity, price_level)?,
            OrderSide::Bid => self.bid_side.fill(fill_quantity, price_level)?,
        };

        for order_id in filled_orders {
            self.order_lookup.remove(&order_id);
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use domain::order::{OrderSide, Timestamp};

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

    fn make_book() -> Book {
        let asset_id = AssetId::new(999);
        let book_config = BookConfig::new(20, 10, 10);
        let bid_side_config = SideConfig::new(5);
        let ask_side_config = SideConfig::new(5);

        Book::new(asset_id, book_config, bid_side_config, ask_side_config)
    }

    #[test]
    fn inspect_side_size() {
        println!("Book: {} bytes", std::mem::size_of::<Book>());
    }

    #[test]
    fn bid_top_of_book_is_correct() {
        let mut book = make_book();

        let bid_order_1 = make_bid_order(Quantity::new(10), Price::new(100), 0);
        let bid_order_2 = make_bid_order(Quantity::new(12), Price::new(105), 1);
        let bid_order_3 = make_bid_order(Quantity::new(8), Price::new(95), 2);
        let bid_order_4 = make_bid_order(Quantity::new(3), Price::new(105), 3);

        book.add_order(bid_order_1);
        book.add_order(bid_order_2);
        book.add_order(bid_order_3);
        book.add_order(bid_order_4);

        let expected_bid_top_of_book = TopOfBook {
            side: OrderSide::Bid,
            price: Some(Price::new(105)),
            quantity: Some(Quantity::new(15))
        };

        assert_eq!(book.bid_top_of_book(), expected_bid_top_of_book);
    }

    #[test]
    fn ask_top_of_book_is_correct() {
        let mut book = make_book();

        let bid_order_1 = make_ask_order(Quantity::new(10), Price::new(100), 0);
        let bid_order_2 = make_ask_order(Quantity::new(12), Price::new(95), 1);
        let bid_order_3 = make_ask_order(Quantity::new(8), Price::new(105), 2);
        let bid_order_4 = make_ask_order(Quantity::new(3), Price::new(95), 3);

        book.add_order(bid_order_1);
        book.add_order(bid_order_2);
        book.add_order(bid_order_3);
        book.add_order(bid_order_4);

        let expected_ask_top_of_book = TopOfBook {
            side: OrderSide::Ask,
            price: Some(Price::new(95)),
            quantity: Some(Quantity::new(15))
        };

        assert_eq!(book.ask_top_of_book(), expected_ask_top_of_book);
    }

    #[test]
    fn quantity_at_is_correct() {
        let mut book = make_book();

        let bid_order_1 = make_bid_order(Quantity::new(10), Price::new(100), 0);
        let bid_order_2 = make_bid_order(Quantity::new(12), Price::new(105), 1);
        let ask_order_1 = make_ask_order(Quantity::new(8), Price::new(90), 2);
        let ask_order_2 = make_ask_order(Quantity::new(3), Price::new(90), 3);

        book.add_order(bid_order_1);
        book.add_order(bid_order_2);
        book.add_order(ask_order_1);
        book.add_order(ask_order_2);

        assert_eq!(book.quantity_at(OrderSide::Bid, Price::new(50)), None);
        assert_eq!(book.quantity_at(OrderSide::Ask, Price::new(50)), None);

        assert_eq!(book.quantity_at(OrderSide::Bid, Price::new(100)), Some(Quantity::new(10)));
        assert_eq!(book.quantity_at(OrderSide::Bid, Price::new(105)), Some(Quantity::new(12)));

        assert_eq!(book.quantity_at(OrderSide::Ask, Price::new(90)), Some(Quantity::new(11)));
    }

    #[test]
    fn add_bid_order_registers_lookup() {
        let mut book = make_book();

        let order = make_bid_order(Quantity::new(10), Price::new(100), 0);
        let id = order.id;

        book.add_order(order);

        assert!(book.order_lookup.contains_key(&id));
        assert_eq!(book.order_lookup.len(), 1);
    }

    #[test]
    fn add_ask_order_registers_lookup() {
        let mut book = make_book();

        let order = make_ask_order(Quantity::new(10), Price::new(100), 0);
        let id = order.id;

        book.add_order(order);

        assert!(book.order_lookup.contains_key(&id));
        assert_eq!(book.order_lookup.len(), 1);
    }

    #[test]
    fn cancel_existing_order_removes_lookup() {
        let mut book = make_book();

        let order = make_bid_order(Quantity::new(10), Price::new(100), 0);
        let id = order.id;

        book.add_order(order);

        assert!(book.order_lookup.contains_key(&id));

        book.cancel_order(id).unwrap();

        assert!(!book.order_lookup.contains_key(&id));
        assert_eq!(book.order_lookup.len(), 0);
    }

    #[test]
    fn cancel_unknown_order_returns_error() {
        let mut book = make_book();

        let result = book.cancel_order(OrderId::new(999));

        assert!(result.is_err());
    }

    #[test]
    fn fill_fully_filled_order_removes_lookup() {
        let mut book = make_book();

        let order = make_bid_order(Quantity::new(10), Price::new(100), 0);
        let id = order.id;

        book.add_order(order);

        assert!(book.order_lookup.contains_key(&id));

        book.fill(
            Quantity::new(10),
            Price::new(100),
            OrderSide::Bid,
        )
        .unwrap();

        assert!(!book.order_lookup.contains_key(&id));
        assert_eq!(book.order_lookup.len(), 0);
    }

    #[test]
    fn fill_partially_filled_order_keeps_lookup() {
        let mut book = make_book();

        let order = make_bid_order(Quantity::new(10), Price::new(100), 0);
        let id = order.id;

        book.add_order(order);

        book.fill(
            Quantity::new(5),
            Price::new(100),
            OrderSide::Bid,
        )
        .unwrap();

        assert!(book.order_lookup.contains_key(&id));
        assert_eq!(book.order_lookup.len(), 1);
    }

    #[test]
    fn multiple_orders_are_tracked() {
        let mut book = make_book();

        let order1 = make_bid_order(Quantity::new(10), Price::new(100), 0);
        let order2 = make_bid_order(Quantity::new(10), Price::new(101), 1);
        let id1 = order1.id;
        let id2 = order2.id;

        book.add_order(order1);
        book.add_order(order2);

        assert!(book.order_lookup.contains_key(&id1));
        assert!(book.order_lookup.contains_key(&id2));
        assert_eq!(book.order_lookup.len(), 2);

        book.cancel_order(id1).unwrap();

        assert!(!book.order_lookup.contains_key(&id1));
        assert!(book.order_lookup.contains_key(&id2));
    }
}