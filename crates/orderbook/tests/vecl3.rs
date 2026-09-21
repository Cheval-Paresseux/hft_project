use orderbook::{
    errors::OrderBookError,
    level3::{L3OrderBook, VecL3OrderBook},
    order::{LimitOrder, OrderSide},
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn limit(id: u64, quantity: u64, side: OrderSide, price: u64) -> LimitOrder {
    LimitOrder::new(id.into(), 0.into(), quantity.into(), side, price.into())
}

// ── Workflow tests: any test simulates an order flow through the book ─────────

#[test]
fn liquidity_build_up_and_queries() {
    let mut book = VecL3OrderBook::default();

    // Market makers add liquidity on both sides, across several levels.
    book.add_order(limit(1, 10, OrderSide::Bid, 100));
    book.add_order(limit(2, 5, OrderSide::Bid, 105));
    book.add_order(limit(3, 20, OrderSide::Bid, 100));
    book.add_order(limit(4, 8, OrderSide::Ask, 200));
    book.add_order(limit(5, 7, OrderSide::Ask, 195));

    // Best level on each side.
    assert_eq!(book.top_price(OrderSide::Bid), Some(105.into()));
    assert_eq!(book.top_price(OrderSide::Ask), Some(195.into()));
    assert_eq!(book.top_order(OrderSide::Bid), Some((2.into(), 105.into(), 5.into())));

    // Aggregated liquidity at a price level.
    assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), 30.into());

    // What the ask side can offer, with and without a price bound.
    assert_eq!(book.quantity_up_to(OrderSide::Ask, Some(195.into())), 7.into());
    assert_eq!(
        book.orders_up_to(OrderSide::Ask, None),
        vec![
            (5.into(), 195.into(), 7.into()),
            (4.into(), 200.into(), 8.into()),
        ]
    );
}

#[test]
fn order_lifecycle() {
    let mut book = VecL3OrderBook::default();

    // An order arrives and is partially filled.
    book.add_order(limit(1, 10, OrderSide::Bid, 100));
    assert_eq!(book.fill_order(1.into(), 4.into()), Ok(()));
    assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), 6.into());

    // The trader adjusts the remaining quantity.
    assert_eq!(book.modify_order(1.into(), 5.into()), Ok(()));
    assert_eq!(book.order(1.into()), Ok((OrderSide::Bid, 100.into(), 5.into())));

    // The order is fully filled and leaves the book.
    assert_eq!(book.fill_order(1.into(), 5.into()), Ok(()));
    assert_eq!(book.top_order(OrderSide::Bid), None);
    assert_eq!(
        book.cancel_order(1.into()),
        Err(OrderBookError::OrderIdNotFound { order_id: 1.into() })
    );

    // Acting on an unknown order is an error.
    assert_eq!(
        book.cancel_order(42.into()),
        Err(OrderBookError::OrderIdNotFound { order_id: 42.into() })
    );
    assert_eq!(
        book.fill_order(42.into(), 1.into()),
        Err(OrderBookError::OrderIdNotFound { order_id: 42.into() })
    );
}

#[test]
fn fifo_orders_at_same_price() {
    let mut book = VecL3OrderBook::default();

    // Three orders rest at the same ask price.
    book.add_order(limit(1, 10, OrderSide::Ask, 100));
    book.add_order(limit(2, 5, OrderSide::Ask, 100));
    book.add_order(limit(3, 7, OrderSide::Ask, 100));

    // They share a level, in arrival order, and the first one is the FIFO-best.
    assert_eq!(
        book.orders_at(OrderSide::Ask, 100.into()),
        Ok(vec![
            (1.into(), 10.into()),
            (2.into(), 5.into()),
            (3.into(), 7.into()),
        ])
    );
    assert_eq!(book.quantity_at(OrderSide::Ask, 100.into()), 22.into());
    assert_eq!(book.top_order(OrderSide::Ask), Some((1.into(), 100.into(), 10.into())));

    // Once the first order is fully consumed, the next one takes its place.
    assert_eq!(book.fill_order(1.into(), 10.into()), Ok(()));
    assert_eq!(book.top_order(OrderSide::Ask), Some((2.into(), 100.into(), 5.into())));
}