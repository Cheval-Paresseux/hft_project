use crate::{level3::{execution::{ExecutionInstruction::{self, Cancel, Modify}, ExecutionPolicy}, message::EngineMessage::{self, New}}, order::{
    LimitOrder, MarketOrder, MarketTimeInForce, Order, StopLimitOrder, StopOrder, TimeInForce,
}};
use orderbook::{
    errors::OrderBookError, level3::L3OrderBook, order::{LimitOrder as BookLimitOrder, OrderId, OrderSide, Price, Quantity},
};

// ── Matching Engine ───────────────────────────────────────────────────────────

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

// ── Message and Execution ─────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    pub fn submit_message(&mut self, message: EngineMessage) {
        let policy = match message {
            EngineMessage::NewOrder(order) => self.new_order_policy(order),
            EngineMessage::CancelOrder{ order_id } => self.cancel_order_policy(order_id),
            EngineMessage::ModifyOrder{ order_id, new_quantity } => self.modify_order_policy(order_id, new_quantity), 
        };

        self.execute(policy);
    }

    fn execute(&mut self, policy: ExecutionPolicy) {
        for instruction in policy.instructions() {
            let result = match instruction {
                ExecutionInstruction::Add(order) => {
                    self.orderbook.add_order(*order);
                    Ok(())
                }
                ExecutionInstruction::Cancel { order_id } => {
                    self.orderbook.cancel_order(*order_id)
                }
                ExecutionInstruction::Modify { order_id, new_quantity } => {
                    self.orderbook.modify_order(*order_id, *new_quantity)
                }
                ExecutionInstruction::Fill {order_id, fill_quantity } => {
                    self.orderbook.fill_order(*order_id, *fill_quantity)
                }
            };
        }
        //todo: what do we do with the Result ? 
    }
}

// ── Execution Policies ────────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    fn new_order_policy(&self, order: Order) -> ExecutionPolicy {
        let mut policy = ExecutionPolicy::new();
        
        policy
    }

    fn cancel_order_policy(&self, order_id: OrderId) -> ExecutionPolicy {
        let mut policy = ExecutionPolicy::new();
        policy.add(ExecutionInstruction::Cancel { order_id });

        policy
    }

    fn modify_order_policy(&self, order_id: OrderId, new_quantity: Quantity) -> ExecutionPolicy {
        let mut policy = ExecutionPolicy::new();
        policy.add(ExecutionInstruction::Modify { order_id, new_quantity });

        policy
    }
}
