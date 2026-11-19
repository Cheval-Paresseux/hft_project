use crate::common::{BookError, OrderMutationsPolicy};

use types::order::{
    fields::{OrderId, OrderSide, Price, Quantity},
    orders::{BookableOrder, Order},
};

// ── L3 Book Update ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum L3BookUpdate<I> {
    Insert(I),
    Remove(OrderId),
    Replace(OrderId, I),
    Fill(OrderId, Quantity),
    Reduce(OrderId, Quantity, OrderMutationsPolicy),
    Increase(OrderId, Quantity, OrderMutationsPolicy),
}

// ── L2 Book View ──────────────────────────────────────────────────────────────

pub trait L2BookView {
    fn best_price(&self, side: OrderSide) -> Option<Price>;

    // ── Quantity Aggregations ────────────────────────────────────────────────

    fn quantity_at(&self, side: OrderSide, price: Price) -> Quantity;
    fn quantity_through(&self, side: OrderSide, bound: Option<Price>) -> Quantity;
}

// ── L3 Book View ──────────────────────────────────────────────────────────────

pub trait L3BookView: L2BookView {
    type Resting: Order;

    type RestingOrders<'a>: Iterator<Item = (OrderSide, Price, &'a Self::Resting)>
    where
        Self: 'a;

    // ── Peek Orders ──────────────────────────────────────────────────────────

    fn order(&self, order_id: OrderId) -> Option<(OrderSide, Price, &Self::Resting)>;
    fn best_order(&self, side: OrderSide) -> Option<(OrderSide, Price, &Self::Resting)>;
    fn best_order_at(&self, side: OrderSide, price: Price) -> Option<(OrderSide, Price, &Self::Resting)>;

    // ── Order Traversal ──────────────────────────────────────────────────────

    fn orders_at(&self, side: OrderSide, price: Price) -> Self::RestingOrders<'_>;
    fn orders_through(&self, side: OrderSide, bound: Option<Price>) -> Self::RestingOrders<'_>;
    fn orders_through_quantity(&self, side: OrderSide, limit: Option<Quantity>) -> Self::RestingOrders<'_>;
}

// ── L3 Order Book ─────────────────────────────────────────────────────────────

pub trait L3OrderBook: L3BookView {
    type Incoming: BookableOrder;

    // ── Book Mutations ────────────────────────────────────────────────────────

    fn insert(&mut self, order: Self::Incoming) -> Result<(), BookError>;
    fn remove(&mut self, order_id: OrderId) -> Result<(), BookError>;
    fn replace(&mut self, old_order_id: OrderId, new_order: Self::Incoming) -> Result<(), BookError>;

    // ── Orders Mutations ──────────────────────────────────────────────────────

    fn fill(&mut self, order_id: OrderId, filled_quantity: Quantity) -> Result<(), BookError>;
    fn reduce(
        &mut self,
        order_id: OrderId,
        remaining_quantity: Quantity,
        policy: OrderMutationsPolicy,
    ) -> Result<(), BookError>;
    fn increase(
        &mut self,
        order_id: OrderId,
        remaining_quantity: Quantity,
        policy: OrderMutationsPolicy,
    ) -> Result<(), BookError>;

    // ── Updates ──────────────────────────────────────────────────────────────

    fn apply(&mut self, update: L3BookUpdate<Self::Incoming>) -> Result<(), BookError> {
        match update {
            L3BookUpdate::Insert(order) => self.insert(order),
            L3BookUpdate::Remove(order_id) => self.remove(order_id),
            L3BookUpdate::Replace(old_order_id, new_order) => self.replace(old_order_id, new_order),
            L3BookUpdate::Fill(order_id, filled_quantity) => self.fill(order_id, filled_quantity),
            L3BookUpdate::Reduce(order_id, remaining_quantity, policy) => {
                self.reduce(order_id, remaining_quantity, policy)
            }
            L3BookUpdate::Increase(order_id, remaining_quantity, policy) => {
                self.increase(order_id, remaining_quantity, policy)
            }
        }
    }
}
