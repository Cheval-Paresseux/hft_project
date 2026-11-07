use book::trigger::{TriggerBook, TriggerBookError, TriggerCondition, VecTriggerBook};

use types::order::{
    fields::{Price, Quantity},
    orders::{Order, RestingLimitOrder},
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn price(value: u64) -> Price {
    Price::new(value)
}

fn quantity(value: u64) -> Quantity {
    Quantity::new(value)
}

fn resting(id: u64, quantity: u64) -> RestingLimitOrder {
    RestingLimitOrder::new(id.into(), quantity.into(), 0.into())
}

fn ref_entries<'a>(orders: impl IntoIterator<Item = (&'a Price, &'a RestingLimitOrder)>) -> Vec<(u64, u64)> {
    orders
        .into_iter()
        .map(|(threshold, order)| (threshold.get(), order.id().get()))
        .collect()
}

fn owned_entries(orders: &[(Price, RestingLimitOrder)]) -> Vec<(u64, u64)> {
    orders
        .iter()
        .map(|(threshold, order)| (threshold.get(), order.id().get()))
        .collect()
}

// ── Integration test ─────────────────────────────────────────────────────────

#[test]
fn trigger_book_lifecycle() {
    let mut book = VecTriggerBook::<Price, RestingLimitOrder>::new(TriggerCondition::AtOrAbove);

    assert_eq!(book.condition(), TriggerCondition::AtOrAbove);
    assert!(book.best_order().is_none());
    assert!(book.best_order_at(&price(100)).is_none());
    assert!(book.orders_through(None).next().is_none());
    assert_eq!(book.quantity_through(None), quantity(0));

    // ── Add orders ────────────────────────────────────────────────────────────

    book.insert(price(105), resting(1, 10)).unwrap();
    book.insert(price(100), resting(2, 5)).unwrap();
    book.insert(price(100), resting(3, 20)).unwrap();
    book.insert(price(110), resting(4, 8)).unwrap();

    assert_eq!(
        book.insert(price(120), resting(2, 1)),
        Err(TriggerBookError::OrderIdAlreadyExists { order_id: 2.into() })
    );

    // ── Inspect the book ──────────────────────────────────────────────────────

    assert_eq!(book.best_order().map(|(threshold, _)| *threshold), Some(price(100)));

    let (threshold, order) = book.best_order().unwrap();
    assert_eq!((threshold.get(), order.id().get()), (100, 2));

    let (threshold, order) = book.best_order_at(&price(100)).unwrap();
    assert_eq!((threshold.get(), order.id().get()), (100, 2));
    assert!(book.best_order_at(&price(101)).is_none());

    assert_eq!(book.quantity_at(&price(100)), quantity(25));
    assert_eq!(book.quantity_at(&price(101)), quantity(0));
    assert_eq!(book.quantity_through(Some(&price(100))), quantity(25));
    assert_eq!(book.quantity_through(None), quantity(43));

    assert_eq!(ref_entries(book.orders_at(&price(100))), vec![(100, 2), (100, 3)]);
    assert_eq!(
        ref_entries(book.orders_through(Some(&price(100)))),
        vec![(100, 2), (100, 3)]
    );
    assert_eq!(
        ref_entries(book.orders_through_quantity(Some(quantity(25)))),
        vec![(100, 2), (100, 3)]
    );

    // ── Replace an order ──────────────────────────────────────────────────────

    book.replace(4.into(), price(110), resting(5, 8)).unwrap();

    assert!(book.order(4.into()).is_none());
    assert_eq!(
        book.order(5.into())
            .map(|(threshold, order)| (threshold.get(), order.quantity().get())),
        Some((110, 8))
    );

    // ── Reduce an order ───────────────────────────────────────────────────────

    book.reduce(3.into(), quantity(30)).unwrap();

    assert_eq!(book.quantity_at(&price(100)), quantity(35));
    assert_eq!(
        book.order(3.into())
            .map(|(threshold, order)| (threshold.get(), order.quantity().get())),
        Some((100, 30))
    );

    // ── Remove an order ───────────────────────────────────────────────────────

    book.remove(2.into()).unwrap();

    assert!(book.order(2.into()).is_none());
    assert_eq!(book.quantity_at(&price(100)), quantity(30));
    assert_eq!(
        book.remove(2.into()),
        Err(TriggerBookError::OrderIdNotFound { order_id: 2.into() })
    );

    // ── Trigger orders ────────────────────────────────────────────────────────

    assert!(book.trigger(price(99)).is_empty());
    assert_eq!(book.orders_through(None).count(), 3);

    let triggered = book.trigger(price(105));
    assert_eq!(owned_entries(&triggered), vec![(100, 3), (105, 1)]);
    assert_eq!(book.best_order().map(|(threshold, _)| *threshold), Some(price(110)));
    assert_eq!(book.quantity_through(None), quantity(8));

    let triggered = book.trigger(price(110));
    assert_eq!(owned_entries(&triggered), vec![(110, 5)]);

    assert!(book.best_order().is_none());
    assert!(book.best_order_at(&price(110)).is_none());
    assert!(book.orders_through(None).next().is_none());
    assert_eq!(book.quantity_through(None), quantity(0));
}
