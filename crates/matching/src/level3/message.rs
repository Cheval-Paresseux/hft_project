use crate::order::Order;
use orderbook::order::{OrderId, Quantity};

// ── Engine Message ────────────────────────────────────────────────────────────

pub enum EngineMessage {
    NewOrder(Order),
    CancelOrder{ order_id: OrderId },
    ModifyOrder{ order_id: OrderId, new_quantity: Quantity }
}