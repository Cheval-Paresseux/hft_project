use orderbook::level3::{common::OrderBookError, L3OrderBook, VecL3OrderBook};

use types::order::{
    fields::{OrderId, OrderSide, Price, Quantity},
    orders::{LimitOrder, Order, RestingLimitOrder},
    TimeInForce,
};

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

fn make_book() -> VecL3OrderBook<LimitOrder, RestingLimitOrder> {
    VecL3OrderBook::default()
}

fn entries(
    orders: Vec<(OrderSide, Price, &RestingLimitOrder)>,
) -> Vec<(OrderSide, Price, OrderId, Quantity)> {
    orders
        .into_iter()
        .map(|(side, price, order)| (side, price, order.id(), order.quantity()))
        .collect()
}

// ── Integration test ─────────────────────────────────────────────────────────

#[test]
fn level3_book_lifecycle() {
    // ── Empty book ────────────────────────────────────────────────────────────

    let mut book = make_book();

    assert_eq!(book.top_price(OrderSide::Bid), None);
    assert!(book.top_order(OrderSide::Bid).is_none());
    assert!(book.orders_at(OrderSide::Bid, 100.into()).is_empty());
    assert!(book.orders_up_to(OrderSide::Bid, None).is_empty());
    assert!(book.orders_up_to_quantity(OrderSide::Bid, None).is_empty());
    assert_eq!(book.quantity_at(OrderSide::Bid, 100.into()), None);
    assert_eq!(book.quantity_up_to(OrderSide::Bid, None), Some(0.into()));

    // ── Add liquidity on both sides ───────────────────────────────────────────

    book.add_order(limit(1, 10, OrderSide::Bid, 100)).unwrap();
    book.add_order(limit(2, 5, OrderSide::Bid, 105)).unwrap();
    book.add_order(limit(3, 20, OrderSide::Bid, 100)).unwrap();
    book.add_order(limit(4, 8, OrderSide::Ask, 200)).unwrap();
    book.add_order(limit(5, 7, OrderSide::Ask, 195)).unwrap();

    assert_eq!(
        book.add_order(limit(1, 1, OrderSide::Bid, 101)),
        Err(OrderBookError::OrderIdAlreadyExists { order_id: 1.into() })
    );

    // ── Best prices and orders ────────────────────────────────────────────────

    assert_eq!(book.top_price(OrderSide::Bid), Some(105.into()));
    assert_eq!(book.top_price(OrderSide::Ask), Some(195.into()));

    assert_eq!(
        book.top_order(OrderSide::Bid).map(|(side, price, order)| (
            side,
            price,
            order.id(),
            order.quantity()
        )),
        Some((
            OrderSide::Bid,
            105.into(),
            OrderId::new(2),
            Quantity::new(5)
        ))
    );
    assert_eq!(
        book.order(1.into()).map(|(side, price, order)| (
            side,
            price,
            order.id(),
            order.quantity()
        )),
        Some((
            OrderSide::Bid,
            100.into(),
            OrderId::new(1),
            Quantity::new(10)
        ))
    );

    // ── Aggregates ────────────────────────────────────────────────────────────

    assert_eq!(
        book.quantity_at(OrderSide::Bid, 100.into()),
        Some(30.into())
    );
    assert_eq!(book.quantity_at(OrderSide::Bid, 101.into()), None);
    assert_eq!(
        book.quantity_up_to(OrderSide::Ask, Some(195.into())),
        Some(7.into())
    );
    assert_eq!(book.quantity_up_to(OrderSide::Ask, None), Some(15.into()));

    // ── Order lookups ─────────────────────────────────────────────────────────

    assert_eq!(
        entries(book.orders_at(OrderSide::Bid, 100.into())),
        vec![
            (
                OrderSide::Bid,
                Price::new(100),
                OrderId::new(1),
                Quantity::new(10)
            ),
            (
                OrderSide::Bid,
                Price::new(100),
                OrderId::new(3),
                Quantity::new(20)
            ),
        ]
    );
    assert!(book.orders_at(OrderSide::Bid, 99.into()).is_empty());

    assert_eq!(
        entries(book.orders_up_to(OrderSide::Bid, None)),
        vec![
            (
                OrderSide::Bid,
                Price::new(105),
                OrderId::new(2),
                Quantity::new(5)
            ),
            (
                OrderSide::Bid,
                Price::new(100),
                OrderId::new(1),
                Quantity::new(10)
            ),
            (
                OrderSide::Bid,
                Price::new(100),
                OrderId::new(3),
                Quantity::new(20)
            ),
        ]
    );
    assert!(book
        .orders_up_to(OrderSide::Bid, Some(106.into()))
        .is_empty());

    assert_eq!(
        entries(book.orders_up_to_quantity(OrderSide::Ask, Some(8.into()))),
        vec![
            (
                OrderSide::Ask,
                Price::new(195),
                OrderId::new(5),
                Quantity::new(7)
            ),
            (
                OrderSide::Ask,
                Price::new(200),
                OrderId::new(4),
                Quantity::new(8)
            ),
        ]
    );
    assert_eq!(
        entries(book.orders_up_to_quantity(OrderSide::Bid, Some(5.into()))),
        vec![(
            OrderSide::Bid,
            Price::new(105),
            OrderId::new(2),
            Quantity::new(5)
        )]
    );

    // ── Modify and fill ───────────────────────────────────────────────────────

    book.modify_order(3.into(), 12.into()).unwrap();
    assert_eq!(
        book.quantity_at(OrderSide::Bid, 100.into()),
        Some(22.into())
    );

    book.fill_order(2.into(), 5.into()).unwrap();
    assert!(book.order(2.into()).is_none());
    assert_eq!(book.top_price(OrderSide::Bid), Some(100.into()));
    assert_eq!(
        book.top_order(OrderSide::Bid)
            .map(|(_, _, order)| order.id()),
        Some(1.into())
    );

    book.fill_order(4.into(), 3.into()).unwrap();
    assert_eq!(book.quantity_at(OrderSide::Ask, 200.into()), Some(5.into()));

    // ── Cancel ────────────────────────────────────────────────────────────────

    book.cancel_order(1.into()).unwrap();
    assert!(book.order(1.into()).is_none());
    assert_eq!(
        book.quantity_at(OrderSide::Bid, 100.into()),
        Some(12.into())
    );
    assert_eq!(
        book.cancel_order(1.into()),
        Err(OrderBookError::OrderIdNotFound { order_id: 1.into() })
    );

    // ── Drain the ask side ────────────────────────────────────────────────────

    book.fill_order(5.into(), 7.into()).unwrap();
    assert_eq!(book.top_price(OrderSide::Ask), Some(200.into()));

    book.fill_order(4.into(), 5.into()).unwrap();
    assert_eq!(book.top_price(OrderSide::Ask), None);
    assert!(book.orders_up_to(OrderSide::Ask, None).is_empty());
    assert_eq!(book.quantity_up_to(OrderSide::Ask, None), Some(0.into()));

    // ── Drain the bid side ────────────────────────────────────────────────────

    book.fill_order(3.into(), 12.into()).unwrap();
    assert_eq!(book.top_price(OrderSide::Bid), None);
    assert!(book.top_order(OrderSide::Bid).is_none());
    assert!(book.orders_at(OrderSide::Bid, 100.into()).is_empty());
    assert_eq!(book.quantity_up_to(OrderSide::Bid, None), Some(0.into()));
}
