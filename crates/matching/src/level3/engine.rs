use super::{
    execution::{ExecutionInstruction, ExecutionPolicy},
    message::EngineMessage,
};

use crate::order::{
    LimitOrder, MarketOrder, MarketTimeInForce, Order, TimeInForce,
};

use orderbook::{
    level3::L3OrderBook,
    order::{OrderId, OrderSide, Price, Quantity},
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
            EngineMessage::CancelOrder { order_id } => self.cancel_order_policy(order_id),
            EngineMessage::ModifyOrder {
                order_id,
                new_quantity,
            } => self.modify_order_policy(order_id, new_quantity),
        };

        self.execute(policy);
    }

    fn execute(&mut self, policy: ExecutionPolicy) {
        for instruction in policy.instructions() {
            let _result = match instruction {
                ExecutionInstruction::Add(order) => {
                    self.orderbook.add_order(*order);
                    Ok(())
                }
                ExecutionInstruction::Cancel { order_id } => self.orderbook.cancel_order(*order_id),
                ExecutionInstruction::Modify {
                    order_id,
                    new_quantity,
                } => self.orderbook.modify_order(*order_id, *new_quantity),
                ExecutionInstruction::Fill {
                    order_id,
                    fill_quantity,
                } => self.orderbook.fill_order(*order_id, *fill_quantity),
            };
        }
    }
}

// ── Execution Policies ────────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    fn new_order_policy(&self, order: Order) -> ExecutionPolicy {
        match order {
            Order::Market(o) => self.market_order_policy(o),
            Order::Limit(o) => self.limit_order_policy(o),

            Order::Stop(_) | Order::StopLimit(_) => ExecutionPolicy::new(),
        }
    }

    fn cancel_order_policy(&self, order_id: OrderId) -> ExecutionPolicy {
        let mut policy = ExecutionPolicy::with_capacity(1);
        policy.add(ExecutionInstruction::Cancel { order_id });

        policy
    }

    fn modify_order_policy(&self, order_id: OrderId, new_quantity: Quantity) -> ExecutionPolicy {
        let mut policy = ExecutionPolicy::with_capacity(1);
        policy.add(ExecutionInstruction::Modify {
            order_id,
            new_quantity,
        });

        policy
    }
}

// ── Matching ───────────────────────────────────────────────────────────────────

impl<OB: L3OrderBook> L3MatchingEngine<OB> {
    fn match_against_book(
        &self,
        side: OrderSide,
        price_bound: Option<Price>,
        mut remaining: Quantity,
    ) -> (ExecutionPolicy, Quantity) {
        let mut policy = ExecutionPolicy::new();

        for (order_id, _, resting) in self.orderbook.orders_up_to(side.opposite(), price_bound) {
            if remaining.is_zero() {
                break;
            }

            let fill = if resting <= remaining {
                resting
            } else {
                remaining
            };

            policy.add(ExecutionInstruction::Fill {
                order_id,
                fill_quantity: fill,
            });
            remaining -= fill;
        }

        (policy, remaining)
    }

    fn limit_order_policy(&self, limit: LimitOrder) -> ExecutionPolicy {
        if limit.tif == TimeInForce::FillOrKill {
            let available = self
                .orderbook
                .quantity_up_to(limit.side.opposite(), Some(limit.price));

            if available < limit.quantity {
                return ExecutionPolicy::new();
            }
        }

        let (mut policy, remaining) =
            self.match_against_book(limit.side, Some(limit.price), limit.quantity);

        let rests = matches!(limit.tif, TimeInForce::Day | TimeInForce::GoodTillCanceled);
        if rests && !remaining.is_zero() {
            policy.add(ExecutionInstruction::Add(limit.into_book(remaining)));
        }

        policy
    }

    fn market_order_policy(&self, market: MarketOrder) -> ExecutionPolicy {
        if market.tif == MarketTimeInForce::FillOrKill {
            let available = self.orderbook.quantity_up_to(market.side.opposite(), None);

            if available < market.quantity {
                return ExecutionPolicy::new();
            }
        }

        let (policy, _unfilled) = self.match_against_book(market.side, None, market.quantity);

        policy
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════
