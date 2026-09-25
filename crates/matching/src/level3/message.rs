use crate::order::Order;
use orderbook::order::{OrderId, Quantity};

// ── Engine Message ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EngineMessage {
    NewOrder(Order),
    CancelOrder{ order_id: OrderId },
    ModifyOrder{ order_id: OrderId, new_quantity: Quantity }
}