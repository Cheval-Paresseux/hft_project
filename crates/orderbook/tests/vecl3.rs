use orderbook::level3::{common::OrderBookError, L3OrderBook, VecL3OrderBook};
use types::order::{LimitOrder, OrderSide, TimeInForce};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn limit(id: u64, quantity: u64, side: OrderSide, price: u64) -> LimitOrder {
    LimitOrder::new(
        id.into(),
        quantity.into(),
        0.into(),
        side,
        price.into(),
        TimeInForce::GoodTillCanceled,
    )
}

// ── Integration tests ─────────────────────────────────────────────────────────

#[test]
fn liquidity_build_up_and_queries() {
    let mut book = VecL3OrderBook::default();

    book.add_order(limit(1, 10, OrderSide::Bid, 100)).unwrap();
    book.add_order(limit(2, 5, OrderSide::Bid, 105)).unwrap();
    book.add_order(limit(3, 20, OrderSide::Bid, 100)).unwrap();
    book.add_order(limit(4, 8, OrderSide::Ask, 200)).unwrap();
    book.add_order(limit(5, 7, OrderSide::Ask, 195)).unwrap();

    assert_eq!(book.top_price(OrderSide::Bid), Some(105.into()));
    assert_eq!(book.top_price(OrderSide::Ask), Some(195.into()));
    assert_eq!(
        book.top_order(OrderSide::Bid),
        Some((2.into(), 105.into(), 5.into()))
    );

    assert_eq!(
        book.quantity_at(OrderSide::Bid, 100.into()),
        Some(30.into())
    );
    assert_eq!(book.quantity_at(OrderSide::Bid, 101.into()), None);

    assert_eq!(
        book.quantity_up_to(OrderSide::Ask, Some(195.into())),
        Some(7.into())
    );
    assert_eq!(
        book.orders_up_to(OrderSide::Ask, None),
        Some(vec![
            (5.into(), 195.into(), 7.into()),
            (4.into(), 200.into(), 8.into()),
        ])
    );
    assert_eq!(book.orders_up_to(OrderSide::Ask, Some(190.into())), None);
}

#[test]
fn order_lifecycle() {
    let mut book = VecL3OrderBook::default();

    book.add_order(limit(1, 10, OrderSide::Bid, 100)).unwrap();
    assert_eq!(book.fill_order(1.into(), 4.into()), Ok(()));
    assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), Some(6.into()));

    assert_eq!(book.modify_order(1.into(), 5.into()), Ok(()));
    assert_eq!(
        book.order(1.into()),
        Some((OrderSide::Bid, 100.into(), 5.into()))
    );

    assert_eq!(book.fill_order(1.into(), 5.into()), Ok(()));
    assert_eq!(book.top_order(OrderSide::Bid), None);
    assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), None);
    assert_eq!(
        book.cancel_order(1.into()),
        Err(OrderBookError::OrderIdNotFound { order_id: 1.into() })
    );

    assert_eq!(
        book.cancel_order(42.into()),
        Err(OrderBookError::OrderIdNotFound {
            order_id: 42.into()
        })
    );
    assert_eq!(
        book.fill_order(42.into(), 1.into()),
        Err(OrderBookError::OrderIdNotFound {
            order_id: 42.into()
        })
    );
}

#[test]
fn fifo_orders_at_same_price() {
    let mut book = VecL3OrderBook::default();

    book.add_order(limit(1, 10, OrderSide::Ask, 100)).unwrap();
    book.add_order(limit(2, 5, OrderSide::Ask, 100)).unwrap();
    book.add_order(limit(3, 7, OrderSide::Ask, 100)).unwrap();

    assert_eq!(
        book.orders_at(OrderSide::Ask, 100.into()),
        Some(vec![
            (1.into(), 10.into()),
            (2.into(), 5.into()),
            (3.into(), 7.into()),
        ])
    );
    assert_eq!(
        book.quantity_at(OrderSide::Ask, 100.into()),
        Some(22.into())
    );
    assert_eq!(
        book.top_order(OrderSide::Ask),
        Some((1.into(), 100.into(), 10.into()))
    );

    assert_eq!(book.fill_order(1.into(), 10.into()), Ok(()));
    assert_eq!(
        book.top_order(OrderSide::Ask),
        Some((2.into(), 100.into(), 5.into()))
    );
}

#[test]
fn duplicate_order_id_is_rejected() {
    let mut book = VecL3OrderBook::default();

    book.add_order(limit(1, 10, OrderSide::Bid, 100)).unwrap();

    assert_eq!(
        book.add_order(limit(1, 5, OrderSide::Bid, 105)),
        Err(OrderBookError::OrderIdAlreadyExists { order_id: 1.into() })
    );

    assert_eq!(
        book.order(1.into()),
        Some((OrderSide::Bid, 100.into(), 10.into()))
    );
    assert_eq!(book.top_price(OrderSide::Bid), Some(100.into()));
    assert_eq!(book.quantity_at(OrderSide::Bid, 105.into()), None);
}
