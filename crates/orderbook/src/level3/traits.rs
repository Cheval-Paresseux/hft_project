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
    fn fill_order(
        &mut self,
        order_id: OrderId,
        fill_quantity: Quantity,
    ) -> Result<(), OrderBookError>;

    fn order(&self, order_id: OrderId) -> Result<(OrderSide, Price, Quantity), OrderBookError>;
    fn top_order(&self, side: OrderSide) -> Option<(OrderId, Price, Quantity)>;
    fn orders_at(
        &self,
        side: OrderSide,
        price: Price,
    ) -> Result<Vec<(OrderId, Quantity)>, OrderBookError>;
    fn orders_up_to(
        &self,
        side: OrderSide,
        bound: Option<Price>,
    ) -> Vec<(OrderId, Price, Quantity)>;
    fn orders_up_to_quantity(
        &self,
        side: OrderSide,
        bound: Option<Quantity>,
    ) -> Vec<(OrderId, Price, Quantity)>;

    fn top_price(&self, side: OrderSide) -> Option<Price>;
    fn quantity_at(&self, side: OrderSide, price: Price) -> Quantity;
    fn quantity_up_to(&self, side: OrderSide, bound: Option<Price>) -> Quantity;
}
