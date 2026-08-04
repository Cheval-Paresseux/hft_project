use crate::{
    errors::OrderBookError,
    order::{LimitOrder, OrderId, OrderSide, Price, Quantity},
};

// ── L3 Order Book  ────────────────────────────────────────────────────────────

pub trait L3OrderBook: Default {
    fn add_order(&mut self, order: LimitOrder);
    fn cancel_order(&mut self, order_id: OrderId) -> Result<(), OrderBookError>;
    fn modify_order(
        &mut self,
        order_id: OrderId,
        new_quantity: Quantity,
    ) -> Result<(), OrderBookError>;

    fn fill(&mut self, order_id: OrderId, fill_quantity: Quantity) -> Result<(), OrderBookError>;
    fn best(&self, side: OrderSide) -> Option<(OrderId, Price, Quantity)>;

    fn best_price(&self, side: OrderSide) -> Option<Price>;
    fn quantity_at(&self, side: OrderSide, price: Price) -> Quantity;
}
