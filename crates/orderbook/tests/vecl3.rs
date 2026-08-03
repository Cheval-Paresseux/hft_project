use orderbook::{
    errors::OrderBookError,
    level3::{L3OrderBook, VecL3OrderBook},
    order::{LimitOrder, OrderSide},
};

// ── Helpers  ──────────────────────────────────────────────────────────────────

fn limit(id: u64, quantity: u64, side: OrderSide, price: u64) -> LimitOrder {
    LimitOrder::new(id.into(), 0.into(), quantity.into(), side, price.into())
}

// ── Test ──────────────────────────────────────────────────────────────────────

#[test]
fn add_order() {
    let mut book = VecL3OrderBook::default();

    book.add_order(limit(1, 10, OrderSide::Bid, 100));
    book.add_order(limit(2, 10, OrderSide::Bid, 105));
    book.add_order(limit(3, 10, OrderSide::Ask, 200));
    book.add_order(limit(4, 10, OrderSide::Ask, 195));

    assert_eq!(book.best_price(OrderSide::Bid), Some(105.into()));
    assert_eq!(book.best_price(OrderSide::Ask), Some(195.into()));
    assert_eq!(
        book.best(OrderSide::Bid),
        Some((2.into(), 105.into(), 10.into()))
    );
    assert_eq!(
        book.best(OrderSide::Ask),
        Some((4.into(), 195.into(), 10.into()))
    );
    assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), 10.into());
    assert_eq!(book.quantity_at(OrderSide::Ask, 200.into()), 10.into());
    assert_eq!(book.quantity_at(OrderSide::Bid, 42.into()), 0.into());
}

#[test]
fn modify_and_fill() {
    let mut book = VecL3OrderBook::default();
    book.add_order(limit(1, 10, OrderSide::Bid, 100));
    book.add_order(limit(2, 5, OrderSide::Bid, 100));

    assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), 15.into());

    assert_eq!(book.modify_order(1.into(), 7.into()), Ok(()));
    assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), 12.into());

    assert_eq!(book.fill(1.into(), 7.into()), Ok(()));
    assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), 5.into());

    assert_eq!(
        book.fill(2.into(), 6.into()),
        Err(OrderBookError::FillExceedsOrderQuantity {
            order_id: 2.into(),
            order_quantity: 5.into(),
            fill_quantity: 6.into(),
        })
    );
}

#[test]
fn cancel_order() {
    let mut book = VecL3OrderBook::default();
    book.add_order(limit(1, 10, OrderSide::Bid, 100));
    book.add_order(limit(2, 10, OrderSide::Ask, 100));

    assert_eq!(book.cancel_order(1.into()), Ok(()));
    assert_eq!(book.best(OrderSide::Bid), None);
    assert_eq!(book.best_price(OrderSide::Bid), None);
    assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), 0.into());

    assert_eq!(
        book.cancel_order(1.into()),
        Err(OrderBookError::OrderIdNotFound { order_id: 1.into() })
    );
}
