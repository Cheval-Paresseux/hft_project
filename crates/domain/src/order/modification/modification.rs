use crate::order::fields::{ ModificationId, OrderId, Timestamp, Quantity, OrderInstruction };

// ── Modification ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Modification {
    pub id: ModificationId,
    pub timestamp: Timestamp,
    pub order_id: OrderId,
    pub instruction: ModificationInstruction,
}

impl Modification {
    pub fn new_reduce_quantity(id: ModificationId, timestamp: Timestamp, order_id: OrderId, new_quantity: Quantity) -> Self {
        Self { id, timestamp, order_id, instruction: ModificationInstruction::ReduceQuantity(new_quantity) }
    }

    pub fn new_increase_quantity(id: ModificationId, timestamp: Timestamp, order_id: OrderId, new_quantity: Quantity) -> Self {
        Self { id, timestamp, order_id, instruction: ModificationInstruction::IncreaseQuantity(new_quantity) }
    }

    pub fn new_change_instruction(id: ModificationId, timestamp: Timestamp, order_id: OrderId, new_instruction: OrderInstruction) -> Self {
        Self { id, timestamp, order_id, instruction: ModificationInstruction::ChangeInstruction(new_instruction) }
    }
}

// ── Modification Instruction ──────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModificationInstruction {
    ReduceQuantity(Quantity),
    IncreaseQuantity(Quantity),
    ChangeInstruction(OrderInstruction),
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_order_size() {
        println!("Modification Instruction: {} bytes", std::mem::size_of::<ModificationInstruction>());
        println!("Modification: {} bytes", std::mem::size_of::<Modification>());
    }
}
