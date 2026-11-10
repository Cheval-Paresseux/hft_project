use book::{
    OrderMutationsPolicy,
    level3::{BookError, L3OrderBook, VecL3OrderBook},
};

use types::order::{
    TimeInForce,
    fields::{OrderId, OrderSide, Price, Quantity},
    orders::{LimitOrder, Order, RestingLimitOrder},
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn price(value: u64) -> Price {
    Price::new(value)
}

fn quantity(value: u64) -> Quantity {
    Quantity::new(value)
}

fn limit(id: u64, quantity: u64, side: OrderSide, price: u64) -> LimitOrder {
    LimitOrder::new(
        id.into(),
        quantity.into(),
        0.into(),
        side,
        Price::new(price),
        TimeInForce::GoodTillCanceled,
    )
}

fn make_book() -> VecL3OrderBook<LimitOrder, RestingLimitOrder> {
    VecL3OrderBook::new(OrderMutationsPolicy::KeepPriority, OrderMutationsPolicy::KeepPriority)
}

fn entries<'a>(
    orders: impl IntoIterator<Item = (OrderSide, Price, &'a RestingLimitOrder)>,
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

    assert!(book.best_order(OrderSide::Bid).is_none());
    assert!(book.best_order_at(OrderSide::Bid, price(100)).is_none());
    assert!(book.orders_at(OrderSide::Bid, price(100)).next().is_none());
    assert!(book.orders_through(OrderSide::Bid, None).next().is_none());
    assert!(book.orders_through_quantity(OrderSide::Bid, None).next().is_none());
    assert_eq!(book.quantity_at(OrderSide::Bid, price(100)), quantity(0));
    assert_eq!(book.quantity_through(OrderSide::Bid, None), quantity(0));

    // ── Insert liquidity on both sides ────────────────────────────────────────

    book.insert(limit(1, 10, OrderSide::Bid, 100)).unwrap();
    book.insert(limit(2, 5, OrderSide::Bid, 105)).unwrap();
    book.insert(limit(3, 20, OrderSide::Bid, 100)).unwrap();
    book.insert(limit(4, 8, OrderSide::Ask, 200)).unwrap();
    book.insert(limit(5, 7, OrderSide::Ask, 195)).unwrap();

    assert_eq!(
        book.insert(limit(1, 1, OrderSide::Bid, 101)),
        Err(BookError::OrderIdAlreadyExists { order_id: 1.into() })
    );

    // ── Best orders and lookups ───────────────────────────────────────────────

    assert_eq!(
        book.best_order(OrderSide::Bid)
            .map(|(side, price, order)| (side, price, order.id(), order.quantity())),
        Some((OrderSide::Bid, price(105), OrderId::new(2), quantity(5)))
    );
    assert_eq!(
        book.best_order(OrderSide::Ask)
            .map(|(side, price, order)| (side, price, order.id(), order.quantity())),
        Some((OrderSide::Ask, price(195), OrderId::new(5), quantity(7)))
    );
    assert_eq!(
        book.order(1.into())
            .map(|(side, price, order)| (side, price, order.id(), order.quantity())),
        Some((OrderSide::Bid, price(100), OrderId::new(1), quantity(10)))
    );
    assert_eq!(
        book.best_order_at(OrderSide::Bid, price(100))
            .map(|(_, _, order)| order.id()),
        Some(1.into())
    );

    // ── Aggregates ────────────────────────────────────────────────────────────

    assert_eq!(book.quantity_at(OrderSide::Bid, price(100)), quantity(30));
    assert_eq!(book.quantity_at(OrderSide::Bid, price(101)), quantity(0));
    assert_eq!(book.quantity_through(OrderSide::Ask, Some(price(195))), quantity(7));
    assert_eq!(book.quantity_through(OrderSide::Ask, None), quantity(15));

    // ── Order traversal ───────────────────────────────────────────────────────

    assert_eq!(
        entries(book.orders_at(OrderSide::Bid, price(100))),
        vec![
            (OrderSide::Bid, price(100), OrderId::new(1), quantity(10)),
            (OrderSide::Bid, price(100), OrderId::new(3), quantity(20)),
        ]
    );
    assert!(book.orders_at(OrderSide::Bid, price(99)).next().is_none());

    assert_eq!(
        entries(book.orders_through(OrderSide::Bid, None)),
        vec![
            (OrderSide::Bid, price(105), OrderId::new(2), quantity(5)),
            (OrderSide::Bid, price(100), OrderId::new(1), quantity(10)),
            (OrderSide::Bid, price(100), OrderId::new(3), quantity(20)),
        ]
    );
    assert!(book.orders_through(OrderSide::Bid, Some(price(106))).next().is_none());

    assert_eq!(
        entries(book.orders_through_quantity(OrderSide::Ask, Some(quantity(8)))),
        vec![
            (OrderSide::Ask, price(195), OrderId::new(5), quantity(7)),
            (OrderSide::Ask, price(200), OrderId::new(4), quantity(8)),
        ]
    );
    assert_eq!(
        entries(book.orders_through_quantity(OrderSide::Bid, Some(quantity(5)))),
        vec![(OrderSide::Bid, price(105), OrderId::new(2), quantity(5))]
    );

    // ── Replace an order ──────────────────────────────────────────────────────

    book.replace(2.into(), limit(2, 6, OrderSide::Bid, 101)).unwrap();

    assert_eq!(
        book.order(2.into())
            .map(|(side, price, order)| (side, price, order.quantity())),
        Some((OrderSide::Bid, price(101), quantity(6)))
    );
    assert_eq!(book.quantity_at(OrderSide::Bid, price(105)), quantity(0));
    assert_eq!(
        book.best_order(OrderSide::Bid)
            .map(|(_, price, order)| (price, order.id())),
        Some((price(101), 2.into()))
    );

    // ── Reduce an order ───────────────────────────────────────────────────────

    book.reduce(3.into(), quantity(12)).unwrap();

    assert_eq!(
        book.order(3.into()).map(|(_, _, order)| order.quantity()),
        Some(quantity(12))
    );
    assert_eq!(book.quantity_at(OrderSide::Bid, price(100)), quantity(22));

    // ── Remove orders ─────────────────────────────────────────────────────────

    book.remove(1.into()).unwrap();

    assert!(book.order(1.into()).is_none());
    assert_eq!(book.quantity_at(OrderSide::Bid, price(100)), quantity(12));
    assert_eq!(
        book.remove(1.into()),
        Err(BookError::OrderIdNotFound { order_id: 1.into() })
    );

    book.remove(2.into()).unwrap();

    assert_eq!(
        book.best_order(OrderSide::Bid)
            .map(|(_, price, order)| (price, order.id())),
        Some((price(100), 3.into()))
    );

    // ── Drain the ask side ────────────────────────────────────────────────────

    book.remove(5.into()).unwrap();
    assert_eq!(
        book.best_order(OrderSide::Ask)
            .map(|(_, price, order)| (price, order.id())),
        Some((price(200), 4.into()))
    );

    book.remove(4.into()).unwrap();
    assert!(book.best_order(OrderSide::Ask).is_none());
    assert!(book.orders_through(OrderSide::Ask, None).next().is_none());
    assert_eq!(book.quantity_through(OrderSide::Ask, None), quantity(0));

    // ── Drain the bid side ────────────────────────────────────────────────────

    book.remove(3.into()).unwrap();
    assert!(book.best_order(OrderSide::Bid).is_none());
    assert!(book.orders_at(OrderSide::Bid, price(100)).next().is_none());
    assert_eq!(book.quantity_through(OrderSide::Bid, None), quantity(0));
}
