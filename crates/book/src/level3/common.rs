use types::order::{
    fields::{OrderId, OrderSide, Price, Quantity},
    orders::{IncomingOrder, Order, OrderError},
};

use thiserror::Error;

// ── Order Error ───────────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum OrderBookError {
    #[error(transparent)]
    OrderError(#[from] OrderError),

    #[error("order with id {order_id:?} not found")]
    OrderIdNotFound { order_id: OrderId },

    #[error("order with id {order_id:?} already exists")]
    OrderIdAlreadyExists { order_id: OrderId },

    #[error("price level {price:?} not found")]
    PriceLevelNotFound { price: Price },
}

// ── L3 Order Book  ────────────────────────────────────────────────────────────

pub trait L3OrderBook {
    type Incoming: IncomingOrder;
    type Resting: Order + From<Self::Incoming>;

    type RestingOrders<'a>: Iterator<Item = (OrderSide, Price, &'a Self::Resting)>
    where
        Self: 'a;

    // ── Book Mutations ────────────────────────────────────────────────────────

    fn insert(&mut self, order: Self::Incoming) -> Result<(), OrderBookError>;
    fn remove(&mut self, order_id: OrderId) -> Result<(), OrderBookError>;

    fn reduce(&mut self, order_id: OrderId, remaining_quantity: Quantity) -> Result<(), OrderBookError>;
    fn replace(&mut self, old_order_id: OrderId, new_order: Self::Incoming) -> Result<(), OrderBookError>;

    // ── Peek Orders ──────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(OrderSide, Price, &Self::Resting)>;
    fn best_order(&self, side: OrderSide) -> Option<(OrderSide, Price, &Self::Resting)>;
    fn best_order_at(&self, side: OrderSide, price: Price) -> Option<(OrderSide, Price, &Self::Resting)>;

    // ── Order Traversal ──────────────────────────────────────────────────────

    fn orders_at(&self, side: OrderSide, price: Price) -> Self::RestingOrders<'_>;
    fn orders_through(&self, side: OrderSide, bound: Option<Price>) -> Self::RestingOrders<'_>;
    fn orders_through_quantity(&self, side: OrderSide, limit: Option<Quantity>) -> Self::RestingOrders<'_>;

    // ── Quantity Aggregations ────────────────────────────────────────────────

    fn quantity_at(&self, side: OrderSide, price: Price) -> Quantity;
    fn quantity_through(&self, side: OrderSide, bound: Option<Price>) -> Quantity;
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════
