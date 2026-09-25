use orderbook::{
    order::{LimitOrder as BookLimitOrder, OrderId, Quantity},
};

// ── Execution Instruction ─────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionInstruction {
    Add(BookLimitOrder),
    Cancel { order_id: OrderId },
    Modify { order_id: OrderId, new_quantity: Quantity },
    Fill { order_id: OrderId, fill_quantity: Quantity },
}

// ── Execution Policy ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExecutionPolicy {
    instructions: Vec<ExecutionInstruction>,
}

impl ExecutionPolicy {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new()
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            instructions: Vec::with_capacity(capacity)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
}

impl ExecutionPolicy {
    pub fn add(mut self, instruction: ExecutionInstruction) {
        self.instructions.push(instruction);
    }

    pub fn instructions(&self) -> &[ExecutionInstruction] {
        &self.instructions
    }
}

impl ExecutionPolicy {
    pub fn filling_quantity(&self) -> Quantity {
        self.instructions
            .iter()
            .filter_map(|instruction| match instruction {
                ExecutionInstruction::Fill { fill_quantity, .. } => Some(*fill_quantity),
                _ => None,
            })
            .fold(Quantity::new(0), |acc, quantity| {
                (acc + quantity).unwrap_or(acc)
            })
    }
}