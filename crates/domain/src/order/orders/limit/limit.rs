use crate::order::errors::{FillError, ModificationError, OrderError};
use crate::order::fields::{ModificationId, OrderId, OrderSide, Price, Quantity, Timestamp};
use crate::order::modification::{Modification, ModificationInstruction};

// ── Limit Order ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LimitOrder {
    pub id: OrderId,
    pub timestamp: Timestamp,
    pub quantity: Quantity,
    pub side: OrderSide,
    pub price: Price,
}

impl LimitOrder {
    pub fn new(
        id: OrderId,
        timestamp: Timestamp,
        quantity: Quantity,
        side: OrderSide,
        price: Price,
    ) -> Self {
        Self {
            id,
            timestamp,
            quantity,
            side,
            price,
        }
    }

    pub fn modify(&mut self, modification: Modification) -> Result<(), OrderError> {
        if modification.order_id != self.id {
            return Err(ModificationError::InvalidOrderId {
                modification_id: modification.id,
                order_id: self.id,
                target_order_id: modification.order_id,
            }
            .into());
        }

        match modification.instruction {
            ModificationInstruction::ReduceQuantity(qty) => {
                self.reduce_quantity(qty, modification.id)
            }
            ModificationInstruction::IncreaseQuantity(qty) => {
                self.increase_quantity(qty, modification.id)
            }

            _ => Err(ModificationError::InvalidInstruction {
                modification_id: modification.id,
                order_id: self.id,
                instruction: modification.instruction,
            }
            .into()),
        }
    }

    pub fn fill(&mut self, fill_quantity: Quantity) -> Result<(), OrderError> {
        if self.quantity < fill_quantity {
            return Err(FillError::FillExceedsQuantity {
                order_id: self.id,
                order_quantity: self.quantity,
                fill_quantity,
            }
            .into());
        }

        self.quantity = self
            .quantity
            .sub(fill_quantity)
            .expect("order quantity underflow in fill");

        Ok(())
    }
}

// ── Modifying Order ───────────────────────────────────────────────────────────

impl LimitOrder {
    fn reduce_quantity(
        &mut self,
        quantity: Quantity,
        modification_id: ModificationId,
    ) -> Result<(), OrderError> {
        if self.quantity == quantity {
            return Err(ModificationError::RedundantModification {
                modification_id,
                order_id: self.id,
            }
            .into());
        } else if self.quantity < quantity {
            return Err(ModificationError::InvalidQuantityForReducing {
                modification_id,
                order_id: self.id,
                order_quantity: self.quantity,
                new_quantity: quantity,
            }
            .into());
        }

        self.quantity = quantity;

        Ok(())
    }

    fn increase_quantity(
        &mut self,
        quantity: Quantity,
        modification_id: ModificationId,
    ) -> Result<(), OrderError> {
        if self.quantity == quantity {
            return Err(ModificationError::RedundantModification {
                modification_id,
                order_id: self.id,
            }
            .into());
        } else if self.quantity > quantity {
            return Err(ModificationError::InvalidQuantityForIncreasing {
                modification_id,
                order_id: self.id,
                order_quantity: self.quantity,
                new_quantity: quantity,
            }
            .into());
        }

        self.quantity = quantity;

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_order_size() {
        println!("Resting Order: {} bytes", std::mem::size_of::<LimitOrder>());
    }

    fn make_order() -> LimitOrder {
        LimitOrder::new(
            OrderId::new(1),
            Timestamp::new(1000),
            Quantity::new(100),
            OrderSide::Bid,
            Price::new(10),
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
    fn fill() {
        let mut order = make_order();
        let fill_quantity = Quantity::new(75);

        assert!(order.fill(fill_quantity).is_ok());
        assert_eq!(order.quantity, Quantity::new(25));
    }

    #[test]
    fn fill_exceeds_quantity() {
        let mut order = make_order();
        let fill_quantity = Quantity::new(101);

        assert!(order.fill(fill_quantity).is_err());
    }

    #[test]
    fn modify_wrong_order_id() {
        let mut order = make_order();
        let modification = make_modification(
            OrderId::new(99),
            ModificationInstruction::ReduceQuantity(Quantity::new(50)),
        );

        assert!(matches!(
            order.modify(modification),
            Err(OrderError::InvalidModification(
                ModificationError::InvalidOrderId { .. }
            ))
        ));
    }

    #[test]
    fn reduce_quantity() {
        let mut order = make_order();
        let modification = make_modification(
            order.id,
            ModificationInstruction::ReduceQuantity(Quantity::new(50)),
        );

        assert!(order.modify(modification).is_ok());
        assert_eq!(order.quantity, Quantity::new(50));
    }

    #[test]
    fn reduce_quantity_same_quantity() {
        let mut order = make_order();
        let modification = make_modification(
            order.id,
            ModificationInstruction::ReduceQuantity(Quantity::new(100)),
        );

        assert!(matches!(
            order.modify(modification),
            Err(OrderError::InvalidModification(
                ModificationError::RedundantModification { .. }
            ))
        ));
    }

    #[test]
    fn reduce_quantity_exceeds_order_quantity() {
        let mut order = make_order();
        let modification = make_modification(
            order.id,
            ModificationInstruction::ReduceQuantity(Quantity::new(150)),
        );

        assert!(matches!(
            order.modify(modification),
            Err(OrderError::InvalidModification(
                ModificationError::InvalidQuantityForReducing { .. }
            ))
        ));
    }

    #[test]
    fn increase_quantity() {
        let mut order = make_order();
        let modification = make_modification(
            order.id,
            ModificationInstruction::IncreaseQuantity(Quantity::new(150)),
        );

        assert!(order.modify(modification).is_ok());
        assert_eq!(order.quantity, Quantity::new(150));
    }

    #[test]
    fn increase_quantity_same_quantity() {
        let mut order = make_order();
        let modification = make_modification(
            order.id,
            ModificationInstruction::IncreaseQuantity(Quantity::new(100)),
        );

        assert!(matches!(
            order.modify(modification),
            Err(OrderError::InvalidModification(
                ModificationError::RedundantModification { .. }
            ))
        ));
    }

    #[test]
    fn increase_quantity_below_order_quantity() {
        let mut order = make_order();
        let modification = make_modification(
            order.id,
            ModificationInstruction::IncreaseQuantity(Quantity::new(50)),
        );

        assert!(matches!(
            order.modify(modification),
            Err(OrderError::InvalidModification(
                ModificationError::InvalidQuantityForIncreasing { .. }
            ))
        ));
    }
}
