use crate::order::{
    LimitOrder, MarketOrder, MarketTimeInForce, Order, StopLimitOrder, StopOrder, TimeInForce,
};
use orderbook::{
    level3::L3OrderBook,
    order::{LimitOrder as BookLimitOrder, OrderId, OrderSide, Price, Quantity},
};

pub struct L3MatchingEngine<OB: L3OrderBook> {
    orderbook: OB,
}

impl<OB: L3OrderBook> Default for L3MatchingEngine<OB> {
    fn default() -> Self {
        Self {
            orderbook: OB::default(),
        }
    }
}
