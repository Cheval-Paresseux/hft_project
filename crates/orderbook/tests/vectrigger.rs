use orderbook::trigger::{TriggerBook, TriggerBookError, TriggerCondition, VecTriggerBook};

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

fn ref_entries(orders: &[(&Price, &RestingLimitOrder)]) -> Vec<(u64, u64)> {
    orders
        .iter()
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

    assert_eq!(book.top_threshold(), None);
    assert!(book.top_order().is_none());
    assert!(book.orders_up_to(None).is_empty());
    assert_eq!(book.quantity_up_to(None), Some(quantity(0)));

    // ── Add orders ────────────────────────────────────────────────────────────

    book.add_order(price(105), resting(1, 10)).unwrap();
    book.add_order(price(100), resting(2, 5)).unwrap();
    book.add_order(price(100), resting(3, 20)).unwrap();
    book.add_order(price(110), resting(4, 8)).unwrap();

    assert_eq!(
        book.add_order(price(120), resting(2, 1)),
        Err(TriggerBookError::OrderIdAlreadyExists { order_id: 2.into() })
    );

    // ── Inspect the book ──────────────────────────────────────────────────────

    assert_eq!(book.top_threshold(), Some(&price(100)));

    let (threshold, order) = book.top_order().unwrap();
    assert_eq!((threshold.get(), order.id().get()), (100, 2));

    assert_eq!(book.quantity_at(&price(100)), Some(quantity(25)));
    assert_eq!(book.quantity_at(&price(101)), None);
    assert_eq!(book.quantity_up_to(Some(&price(100))), Some(quantity(25)));
    assert_eq!(book.quantity_up_to(None), Some(quantity(43)));

    assert_eq!(
        ref_entries(&book.orders_at(&price(100))),
        vec![(100, 2), (100, 3)]
    );
    assert_eq!(
        ref_entries(&book.orders_up_to(Some(&price(100)))),
        vec![(100, 2), (100, 3)]
    );
    assert_eq!(
        ref_entries(&book.orders_up_to_quantity(Some(quantity(25)))),
        vec![(100, 2), (100, 3)]
    );

    // ── Modify an order ───────────────────────────────────────────────────────

    book.modify_order(3.into(), quantity(30)).unwrap();

    assert_eq!(book.quantity_at(&price(100)), Some(quantity(35)));
    assert_eq!(
        book.order(3.into())
            .map(|(threshold, order)| (threshold.get(), order.quantity().get())),
        Some((100, 30))
    );

    // ── Cancel an order ───────────────────────────────────────────────────────

    book.cancel_order(2.into()).unwrap();

    assert!(book.order(2.into()).is_none());
    assert_eq!(book.quantity_at(&price(100)), Some(quantity(30)));
    assert_eq!(
        book.cancel_order(2.into()),
        Err(TriggerBookError::OrderIdNotFound { order_id: 2.into() })
    );

    // ── Trigger orders ────────────────────────────────────────────────────────

    assert!(book.trigger_orders(&price(99)).is_empty());
    assert_eq!(book.orders_up_to(None).len(), 3);

    let triggered = book.trigger_orders(&price(105));
    assert_eq!(owned_entries(&triggered), vec![(100, 3), (105, 1)]);
    assert_eq!(book.top_threshold(), Some(&price(110)));
    assert_eq!(book.quantity_up_to(None), Some(quantity(8)));

    let triggered = book.trigger_orders(&price(110));
    assert_eq!(owned_entries(&triggered), vec![(110, 4)]);

    assert_eq!(book.top_threshold(), None);
    assert!(book.top_order().is_none());
    assert!(book.orders_up_to(None).is_empty());
    assert_eq!(book.quantity_up_to(None), Some(quantity(0)));
}
