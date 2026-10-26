use orderbook::trigger::{Direction, TriggerBook, TriggerBookError, VecTriggerBook};
use types::order::{
    fields::Price,
    orders::{Order, RestingLimitOrder},
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn price(value: u64) -> Price {
    Price::new(value)
}

fn order(id: u64, quantity: u64) -> RestingLimitOrder {
    RestingLimitOrder::new(id.into(), quantity.into(), 0.into())
}

// ── Integration tests ─────────────────────────────────────────────────────────

#[test]
fn increasing_book_fires_fifo_within_equal_triggers() {
    let mut book = VecTriggerBook::new(Direction::Increasing);

    book.add_order(price(105), order(1, 10)).unwrap();
    book.add_order(price(100), order(2, 5)).unwrap();
    book.add_order(price(100), order(3, 7)).unwrap();

    assert_eq!(book.top_trigger(), Some(&price(100)));
    assert!(!book.is_triggered(&price(99)));
    assert!(book.is_triggered(&price(100)));

    let triggered = book.pop_triggered(&price(100));
    assert_eq!(triggered.len(), 2);
    assert_eq!(triggered[0].1.id(), 2.into());
    assert_eq!(triggered[1].1.id(), 3.into());

    assert_eq!(
        book.top_order()
            .map(|(trigger, order)| (*trigger, order.id())),
        Some((price(105), 1.into()))
    );
}

#[test]
fn decreasing_book_fires_on_falling_price() {
    let mut book = VecTriggerBook::new(Direction::Decreasing);

    book.add_order(price(100), order(1, 10)).unwrap();
    book.add_order(price(110), order(2, 5)).unwrap();
    book.add_order(price(105), order(3, 7)).unwrap();

    assert_eq!(book.top_trigger(), Some(&price(110)));
    assert!(!book.is_triggered(&price(111)));
    assert_eq!(
        book.orders_triggered(&price(105))
            .unwrap()
            .iter()
            .map(|(_, order)| order.id())
            .collect::<Vec<_>>(),
        vec![2.into(), 3.into()]
    );

    let triggered = book.pop_triggered(&price(105));
    assert_eq!(triggered.len(), 2);
    assert_eq!(book.top_trigger(), Some(&price(100)));
}

#[test]
fn cancel_modify_and_duplicate_ids() {
    let mut book = VecTriggerBook::new(Direction::Increasing);

    book.add_order(price(100), order(1, 10)).unwrap();

    assert_eq!(
        book.add_order(price(105), order(1, 5)),
        Err(TriggerBookError::OrderIdAlreadyExists { order_id: 1.into() })
    );

    assert_eq!(book.modify_order(1.into(), 4.into()), Ok(()));
    assert_eq!(book.order(1.into()).unwrap().1.quantity(), 4.into());

    assert_eq!(book.cancel_order(1.into()), Ok(()));
    assert!(book.is_empty());
    assert_eq!(
        book.cancel_order(1.into()),
        Err(TriggerBookError::OrderIdNotFound { order_id: 1.into() })
    );
}
