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

#[test]
fn at_or_below_build_up_and_queries() {
    let mut book = trigger_book(TriggerCondition::AtOrBelow);

    book.add_order(price(100), resting(1, 10)).unwrap();
    book.add_order(price(110), resting(2, 5)).unwrap();
    book.add_order(price(105), resting(3, 20)).unwrap();
    book.add_order(price(105), resting(4, 8)).unwrap();

    assert_eq!(book.top_threshold(), Some(&price(110)));

    let (threshold, order) = book.top_order().unwrap();
    assert_eq!((threshold.get(), order.id().get()), (110, 2));

    assert_eq!(book.quantity_at(&price(105)), Some(28.into()));
    assert_eq!(book.quantity_at(&price(104)), None);

    assert_eq!(book.quantity_up_to(Some(&price(105))), Some(33.into()));
    assert_eq!(book.quantity_up_to(Some(&price(111))), Some(0.into()));
    assert_eq!(book.quantity_up_to(None), Some(43.into()));

    assert_eq!(
        ref_entries(&book.orders_at(&price(105))),
        vec![(105, 3), (105, 4)]
    );
    assert_eq!(
        ref_entries(&book.orders_up_to(Some(&price(105)))),
        vec![(105, 3), (105, 4), (110, 2)]
    );
    assert!(book.orders_up_to(Some(&price(111))).is_empty());

    assert_eq!(
        ref_entries(&book.orders_up_to_quantity(Some(5.into()))),
        vec![(110, 2)]
    );
}

#[test]
fn order_lifecycle() {
    let mut book = trigger_book(TriggerCondition::AtOrAbove);

    book.add_order(price(100), resting(1, 10)).unwrap();

    assert_eq!(book.modify_order(1.into(), 4.into()), Ok(()));

    let (threshold, order) = book.order(1.into()).unwrap();
    assert_eq!((threshold.get(), order.quantity().get()), (100, 4));

    assert_eq!(book.cancel_order(1.into()), Ok(()));
    assert!(book.top_order().is_none());
    assert!(book.order(1.into()).is_none());

    assert_eq!(
        book.cancel_order(1.into()),
        Err(TriggerBookError::OrderIdNotFound { order_id: 1.into() })
    );
    assert_eq!(
        book.cancel_order(42.into()),
        Err(TriggerBookError::OrderIdNotFound {
            order_id: 42.into()
        })
    );
    assert_eq!(
        book.modify_order(42.into(), 1.into()),
        Err(TriggerBookError::OrderIdNotFound {
            order_id: 42.into()
        })
    );
}

#[test]
fn at_or_above_triggers_on_rising_price() {
    let mut book = trigger_book(TriggerCondition::AtOrAbove);

    book.add_order(price(100), resting(1, 10)).unwrap();
    book.add_order(price(105), resting(2, 5)).unwrap();
    book.add_order(price(100), resting(3, 7)).unwrap();
    book.add_order(price(110), resting(4, 8)).unwrap();

    assert!(book.trigger_orders(&price(99)).is_empty());

    let triggered = book.trigger_orders(&price(100));
    assert_eq!(owned_entries(&triggered), vec![(100, 1), (100, 3)]);
    assert_eq!(book.quantity_at(&price(100)), None);
    assert_eq!(book.top_threshold(), Some(&price(105)));

    let triggered = book.trigger_orders(&price(110));
    assert_eq!(owned_entries(&triggered), vec![(105, 2), (110, 4)]);
    assert!(book.top_order().is_none());
}

#[test]
fn at_or_below_triggers_on_falling_price() {
    let mut book = trigger_book(TriggerCondition::AtOrBelow);

    book.add_order(price(110), resting(1, 10)).unwrap();
    book.add_order(price(105), resting(2, 5)).unwrap();
    book.add_order(price(100), resting(3, 7)).unwrap();
    book.add_order(price(110), resting(4, 8)).unwrap();

    assert!(book.trigger_orders(&price(111)).is_empty());

    let triggered = book.trigger_orders(&price(110));
    assert_eq!(owned_entries(&triggered), vec![(110, 1), (110, 4)]);
    assert_eq!(book.quantity_at(&price(110)), None);
    assert_eq!(book.top_threshold(), Some(&price(105)));

    let triggered = book.trigger_orders(&price(90));
    assert_eq!(owned_entries(&triggered), vec![(100, 3), (105, 2)]);
    assert!(book.top_order().is_none());
}

#[test]
fn fifo_orders_at_same_threshold() {
    let mut above = trigger_book(TriggerCondition::AtOrAbove);

    above.add_order(price(100), resting(1, 10)).unwrap();
    above.add_order(price(100), resting(2, 5)).unwrap();
    above.add_order(price(100), resting(3, 7)).unwrap();

    assert_eq!(
        ref_entries(&above.orders_at(&price(100))),
        vec![(100, 1), (100, 2), (100, 3)]
    );
    assert_eq!(above.quantity_at(&price(100)), Some(22.into()));

    let (_, top) = above.top_order().unwrap();
    assert_eq!(top.id().get(), 1);

    let triggered = above.trigger_orders(&price(100));
    assert_eq!(
        owned_entries(&triggered),
        vec![(100, 1), (100, 2), (100, 3)]
    );

    let mut below = trigger_book(TriggerCondition::AtOrBelow);

    below.add_order(price(100), resting(1, 10)).unwrap();
    below.add_order(price(100), resting(2, 5)).unwrap();
    below.add_order(price(100), resting(3, 7)).unwrap();

    assert_eq!(
        ref_entries(&below.orders_at(&price(100))),
        vec![(100, 1), (100, 2), (100, 3)]
    );
    assert_eq!(below.quantity_at(&price(100)), Some(22.into()));

    let (_, top) = below.top_order().unwrap();
    assert_eq!(top.id().get(), 1);

    let triggered = below.trigger_orders(&price(100));
    assert_eq!(
        owned_entries(&triggered),
        vec![(100, 1), (100, 2), (100, 3)]
    );
}

#[test]
fn duplicate_order_id_is_rejected() {
    let mut book = trigger_book(TriggerCondition::AtOrAbove);

    book.add_order(price(100), resting(1, 10)).unwrap();

    assert_eq!(
        book.add_order(price(105), resting(1, 5)),
        Err(TriggerBookError::OrderIdAlreadyExists { order_id: 1.into() })
    );

    let (threshold, order) = book.order(1.into()).unwrap();
    assert_eq!((threshold.get(), order.quantity().get()), (100, 10));

    assert_eq!(book.top_threshold(), Some(&price(100)));
    assert_eq!(book.quantity_at(&price(105)), None);
}
