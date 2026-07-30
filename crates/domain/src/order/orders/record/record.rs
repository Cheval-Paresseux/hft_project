use crate::order::modification::{ Modification, ModificationInstruction };
use crate::order::fields::{ ClientOrderId, OrderId, ModificationId, Timestamp, Quantity, OrderSide, OrderInstruction };
use crate::order::errors::{ OrderError, ModificationError };

// ── Record Order ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RecordOrder {
    pub client_id: ClientOrderId,
    pub id: OrderId,
    pub timestamp: Timestamp,
    pub quantity: Quantity,
    pub side: OrderSide,
    pub instruction: OrderInstruction,
}

impl RecordOrder {
    pub fn new(
        client_id: ClientOrderId,
        id: OrderId, 
        timestamp: Timestamp, 
        quantity: Quantity, 
        side: OrderSide,
        instruction: OrderInstruction,
    ) -> Self {
        Self{ client_id, id, timestamp, quantity, side, instruction }
    }

    pub fn modify(&mut self, modification: Modification) -> Result<(), OrderError> {
        if modification.order_id != self.id {
            let error = Err(ModificationError::InvalidOrderId { 
                modification_id: modification.id, order_id: self.id, target_order_id: modification.order_id 
            }.into());

            return error;
        }

        match modification.instruction {
            ModificationInstruction::ReduceQuantity(qty) => self.reduce_quantity(qty, modification.id),
            ModificationInstruction::IncreaseQuantity(qty) => self.increase_quantity(qty, modification.id),
            ModificationInstruction::ChangeInstruction(ins) => self.change_instruction(ins, modification.id),

            // _ => Err(ModificationError::InvalidInstruction { 
            //     modification_id: modification.id, order_id: self.id, instruction: modification.instruction
            // }.into())
        }
    }
}

// ── Modifying Order ───────────────────────────────────────────────────────────

impl RecordOrder {
    fn reduce_quantity(&mut self, quantity: Quantity, modification_id: ModificationId) -> Result<(), OrderError> {
        if self.quantity == quantity {
            return Err(ModificationError::RedundantModification { modification_id, order_id: self.id }.into());
        } else if self.quantity < quantity {
            return Err(ModificationError::InvalidQuantityForReducing {
                modification_id, order_id: self.id, order_quantity: self.quantity, new_quantity: quantity
            }.into());
        }
        
        self.quantity = quantity;
        Ok(())
    }

    fn increase_quantity(&mut self, quantity: Quantity, modification_id: ModificationId) -> Result<(), OrderError> {
        if self.quantity == quantity {
            return Err(ModificationError::RedundantModification { modification_id, order_id: self.id }.into());
        } else if self.quantity > quantity {
            return Err(ModificationError::InvalidQuantityForIncreasing {
                modification_id, order_id: self.id, order_quantity: self.quantity, new_quantity: quantity
            }.into());
        }
        
        self.quantity = quantity;
        Ok(())
    }

    fn change_instruction(&mut self, instruction: OrderInstruction, modification_id: ModificationId) -> Result<(), OrderError> {
        if self.instruction == instruction {
            return Err(ModificationError::RedundantModification { modification_id, order_id: self.id }.into());
        }
        
        self.instruction = instruction;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::fields::{ MarketTimeInForce, LimitTimeInForce, Price };

    #[test]
    fn inspect_order_size() {
        println!("Record Order: {} bytes", std::mem::size_of::<RecordOrder>());
    }

    fn make_order() -> RecordOrder {
        RecordOrder::new(
            ClientOrderId::new("abc").expect("REASON"),
            OrderId::new(1),
            Timestamp::new(1000),
            Quantity::new(100),
            OrderSide::Bid,
            OrderInstruction::Market { tif: MarketTimeInForce::FillOrKill }
        )
    }

    fn make_modification(order_id: OrderId, instruction: ModificationInstruction) -> Modification {
        Modification {
            id: ModificationId::new(1),
            timestamp: Timestamp::new(2000),
            order_id,
            instruction,
        }
    }

    #[test]
    fn modify_wrong_order_id() {
        let mut order = make_order();
        let modification = make_modification(OrderId::new(99), ModificationInstruction::ReduceQuantity(Quantity::new(50)));

        assert!(matches!(order.modify(modification), Err(OrderError::InvalidModification(ModificationError::InvalidOrderId { .. }))));
    }

    #[test]
    fn reduce_quantity() {
        let mut order = make_order();
        let modification = make_modification(order.id, ModificationInstruction::ReduceQuantity(Quantity::new(50)));

        assert!(order.modify(modification).is_ok());
        assert_eq!(order.quantity, Quantity::new(50));
    }

    #[test]
    fn reduce_quantity_same_quantity() {
        let mut order = make_order();
        let modification = make_modification(order.id, ModificationInstruction::ReduceQuantity(Quantity::new(100)));

        assert!(matches!(order.modify(modification), Err(OrderError::InvalidModification(ModificationError::RedundantModification { .. }))));
    }

    #[test]
    fn reduce_quantity_exceeds_order_quantity() {
        let mut order = make_order();
        let modification = make_modification(order.id, ModificationInstruction::ReduceQuantity(Quantity::new(150)));

        assert!(matches!(order.modify(modification), Err(OrderError::InvalidModification(ModificationError::InvalidQuantityForReducing { .. }))));
    }

    #[test]
    fn increase_quantity() {
        let mut order = make_order();
        let modification = make_modification(order.id, ModificationInstruction::IncreaseQuantity(Quantity::new(150)));
        
        assert!(order.modify(modification).is_ok());
        assert_eq!(order.quantity, Quantity::new(150));
    }

    #[test]
    fn increase_quantity_same_quantity() {
        let mut order = make_order();
        let modification = make_modification(order.id, ModificationInstruction::IncreaseQuantity(Quantity::new(100)));
        
        assert!(matches!(order.modify(modification), Err(OrderError::InvalidModification(ModificationError::RedundantModification { .. }))));
    }

    #[test]
    fn increase_quantity_below_order_quantity() {
        let mut order = make_order();
        let modification = make_modification(order.id, ModificationInstruction::IncreaseQuantity(Quantity::new(50)));
        
        assert!(matches!(order.modify(modification), Err(OrderError::InvalidModification(ModificationError::InvalidQuantityForIncreasing { .. }))));
    }

    #[test]
    fn change_instruction() {
        let mut order = make_order();
        let modification = make_modification(order.id, ModificationInstruction::ChangeInstruction(OrderInstruction::Limit { limit: Price::new(100), tif: LimitTimeInForce::GoodTillCancel }));

        assert!(order.modify(modification).is_ok());
        assert_eq!(order.instruction, OrderInstruction::Limit { limit: Price::new(100), tif: LimitTimeInForce::GoodTillCancel });
    }

    #[test]
    fn change_instruction_same_instruction() {
        let mut order = make_order();
        let modification = make_modification(order.id, ModificationInstruction::ChangeInstruction(OrderInstruction::Market { tif: MarketTimeInForce::FillOrKill }));

        assert!(matches!(order.modify(modification), Err(OrderError::InvalidModification(ModificationError::RedundantModification { .. }))));
    }
}