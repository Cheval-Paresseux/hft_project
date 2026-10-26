use types::order::{
    fields::{OrderId, OrderSide, Price, Quantity},
    orders::{LimitOrder, OrderError},
};

use thiserror::Error;

// ── Order Error ───────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum OrderBookError {
    #[error("order error")]
    OrderError(OrderError),

    #[error("order with id {order_id:?} not found")]
    OrderIdNotFound { order_id: OrderId },

    #[error("price level {price:?} not found")]
    PriceLevelNotFound { price: Price },

    #[error("quantity overflow")]
    QuantityOverflow,
}

impl From<OrderError> for OrderBookError {
    fn from(error: OrderError) -> Self {
        Self::OrderError(error)
    }
}

// ── Quantity Overflow not panic ───────────────────────────────────────────────

pub fn add_quantities(rhs: Quantity, lhs: Quantity) -> Result<Quantity, OrderBookError> {
    (rhs + lhs).ok_or(OrderBookError::QuantityOverflow)
}

pub fn sub_quantities(rhs: Quantity, lhs: Quantity) -> Result<Quantity, OrderBookError> {
    (rhs - lhs).ok_or(OrderBookError::QuantityOverflow)
}

// ── L3 Order Book  ────────────────────────────────────────────────────────────

pub trait L3OrderBook: Default {
    // ── Book Mutations ────────────────────────────────────────────────────────────

    fn add_order(&mut self, order: LimitOrder) -> Result<(), OrderBookError>;
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

    // ── Orders Look Up  ───────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(OrderSide, Price, Quantity)>;
    fn top_order(&self, side: OrderSide) -> Option<(OrderId, Price, Quantity)>;
    fn orders_at(&self, side: OrderSide, price: Price) -> Option<Vec<(OrderId, Quantity)>>;
    fn orders_up_to(
        &self,
        side: OrderSide,
        bound: Option<Price>,
    ) -> Option<Vec<(OrderId, Price, Quantity)>>;
    fn orders_up_to_quantity(
        &self,
        side: OrderSide,
        bound: Option<Quantity>,
    ) -> Option<Vec<(OrderId, Price, Quantity)>>;

    // ── Aggregation Look Up ───────────────────────────────────────────────────────

    fn top_price(&self, side: OrderSide) -> Option<Price>;
    fn quantity_at(&self, side: OrderSide, price: Price) -> Option<Quantity>;
    fn quantity_up_to(&self, side: OrderSide, bound: Option<Price>) -> Option<Quantity>;
}
