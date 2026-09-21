use orderbook::{
    order::{LimitOrder as BookLimitOrder, OrderId, OrderSide, Price, Quantity},
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
        Self::default()
    }

    /// Append an instruction, builder style.
    pub fn with(mut self, instruction: ExecutionInstruction) -> Self {
        self.instructions.push(instruction);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn instructions(&self) -> &[ExecutionInstruction] {
        &self.instructions
    }

    /// Total quantity consumed from the book by this policy's fill instructions.
    pub fn filled_quantity(&self) -> Quantity {
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