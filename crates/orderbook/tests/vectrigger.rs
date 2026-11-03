use orderbook::trigger::{TriggerBook, TriggerBookError, TriggerCondition, VecTriggerBook};

use types::order::{
    fields::Price,
    orders::{Order, RestingLimitOrder},
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn price(value: u64) -> Price {
    Price::new(value)
}

fn resting(id: u64, quantity: u64) -> RestingLimitOrder {
    RestingLimitOrder::new(id.into(), quantity.into(), 0.into())
}

fn trigger_book(condition: TriggerCondition) -> VecTriggerBook<Price, RestingLimitOrder> {
    VecTriggerBook::new(condition)
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

// ── Integration tests ─────────────────────────────────────────────────────────

#[test]
fn at_or_above_build_up_and_queries() {
    let mut book = trigger_book(TriggerCondition::AtOrAbove);

    book.add_order(price(105), resting(1, 10)).unwrap();
    book.add_order(price(100), resting(2, 5)).unwrap();
    book.add_order(price(100), resting(3, 20)).unwrap();
    book.add_order(price(110), resting(4, 8)).unwrap();

    assert_eq!(book.top_threshold(), Some(&price(100)));

    let (threshold, order) = book.top_order().unwrap();
    assert_eq!((threshold.get(), order.id().get()), (100, 2));

    assert_eq!(book.quantity_at(&price(100)), Some(25.into()));
    assert_eq!(book.quantity_at(&price(101)), None);

    assert_eq!(book.quantity_up_to(Some(&price(100))), Some(25.into()));
    assert_eq!(book.quantity_up_to(None), Some(43.into()));

    assert_eq!(
        ref_entries(&book.orders_at(&price(100))),
        vec![(100, 2), (100, 3)]
    );
    assert_eq!(
        ref_entries(&book.orders_up_to(Some(&price(100)))),
        vec![(100, 2), (100, 3)]
    );
    assert!(book.orders_up_to(Some(&price(99))).is_empty());

    assert_eq!(
        ref_entries(&book.orders_up_to_quantity(Some(10.into()))),
        vec![(100, 2), (100, 3)]
    );
}
