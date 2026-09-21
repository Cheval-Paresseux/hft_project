use super::fields::{MarketTimeInForce, TimeInForce};
use orderbook::order::{OrderId, OrderSide, Price, Quantity, Timestamp};

// ── Order ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Order {
    Market(MarketOrder),
    Limit(LimitOrder),
    Stop(StopOrder),
    StopLimit(StopLimitOrder),
}

impl Order {
    pub fn id(&self) -> OrderId {
        match self {
            Order::Market(order) => order.id,
            Order::Limit(order) => order.id,
            Order::Stop(order) => order.id,
            Order::StopLimit(order) => order.id,
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        match self {
            Order::Market(order) => order.timestamp,
            Order::Limit(order) => order.timestamp,
            Order::Stop(order) => order.timestamp,
            Order::StopLimit(order) => order.timestamp,
        }
    }

    pub fn quantity(&self) -> Quantity {
        match self {
            Order::Market(order) => order.quantity,
            Order::Limit(order) => order.quantity,
            Order::Stop(order) => order.quantity,
            Order::StopLimit(order) => order.quantity,
        }
    }

    pub fn side(&self) -> OrderSide {
        match self {
            Order::Market(order) => order.side,
            Order::Limit(order) => order.side,
            Order::Stop(order) => order.side,
            Order::StopLimit(order) => order.side,
        }
    }
}

// ── Market Order ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MarketOrder {
    pub id: OrderId,
    pub timestamp: Timestamp,
    pub quantity: Quantity,
    pub side: OrderSide,
    pub tif: MarketTimeInForce,
}

impl MarketOrder {
    pub fn new(
        id: OrderId,
        timestamp: Timestamp,
        quantity: Quantity,
        side: OrderSide,
        tif: MarketTimeInForce,
    ) -> Self {
        Self {
            id,
            timestamp,
            quantity,
            side,
            tif,
        }
    }
}

// ── Limit Order ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LimitOrder {
    pub id: OrderId,
    pub timestamp: Timestamp,
    pub quantity: Quantity,
    pub side: OrderSide,
    pub price: Price,
    pub tif: TimeInForce,
}

impl LimitOrder {
    pub fn new(
        id: OrderId,
        timestamp: Timestamp,
        quantity: Quantity,
        side: OrderSide,
        price: Price,
        tif: TimeInForce,
    ) -> Self {
        Self {
            id,
            timestamp,
            quantity,
            side,
            price,
            tif,
        }
    }

    pub fn into_book(self, quantity: Quantity) -> orderbook::order::LimitOrder {
        orderbook::order::LimitOrder::new(self.id, self.timestamp, quantity, self.side, self.price)
    }
}

// ── Stop Order ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StopOrder {
    pub id: OrderId,
    pub timestamp: Timestamp,
    pub quantity: Quantity,
    pub side: OrderSide,
    pub trigger: Price,
    pub tif: MarketTimeInForce,
}

impl StopOrder {
    pub fn new(
        id: OrderId,
        timestamp: Timestamp,
        quantity: Quantity,
        side: OrderSide,
        trigger: Price,
        tif: MarketTimeInForce,
    ) -> Self {
        Self {
            id,
            timestamp,
            quantity,
            side,
            trigger,
            tif,
        }
    }
}

impl From<StopOrder> for MarketOrder {
    fn from(order: StopOrder) -> Self {
        Self {
            id: order.id,
            timestamp: order.timestamp,
            quantity: order.quantity,
            side: order.side,
            tif: order.tif,
        }
    }
}

// ── Stop Limit Order ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StopLimitOrder {
    pub id: OrderId,
    pub timestamp: Timestamp,
    pub quantity: Quantity,
    pub side: OrderSide,
    pub trigger: Price,
    pub limit: Price,
    pub tif: TimeInForce,
}

impl StopLimitOrder {
    pub fn new(
        id: OrderId,
        timestamp: Timestamp,
        quantity: Quantity,
        side: OrderSide,
        trigger: Price,
        limit: Price,
        tif: TimeInForce,
    ) -> Self {
        Self {
            id,
            timestamp,
            quantity,
            side,
            trigger,
            limit,
            tif,
        }
    }
}

impl From<StopLimitOrder> for LimitOrder {
    fn from(order: StopLimitOrder) -> Self {
        Self {
            id: order.id,
            timestamp: order.timestamp,
            quantity: order.quantity,
            side: order.side,
            price: order.limit,
            tif: order.tif,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════
