use crate::order::fields::{ ModificationId, OrderId, Quantity };
use crate::order::modification::ModificationInstruction;
use thiserror::Error;

// ── Modification Error ────────────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ModificationError {
    #[error("order id {order_id:?} does not match the targetted order id {target_order_id:?} for modification {modification_id:?}")]
    InvalidOrderId { modification_id: ModificationId, order_id: OrderId, target_order_id: OrderId },

    #[error("order {order_id:?} does not accept {instruction:?} as instruction for modification {modification_id:?}")]
    InvalidInstruction { modification_id: ModificationId, order_id: OrderId, instruction: ModificationInstruction },

    #[error("order {order_id:?} already has the requested value for modification {modification_id:?}")]
    RedundantModification { modification_id: ModificationId, order_id: OrderId },

    #[error("new quantity {new_quantity:?} must be less than current {order_quantity:?} for modification {modification_id:?} on order {order_id:?}")]
    InvalidQuantityForReducing { modification_id: ModificationId, order_id: OrderId, order_quantity: Quantity, new_quantity: Quantity },

    #[error("new quantity {new_quantity:?} must be greater than current {order_quantity:?} for modification {modification_id:?} on order {order_id:?}")]
    InvalidQuantityForIncreasing { modification_id: ModificationId, order_id: OrderId, order_quantity: Quantity, new_quantity: Quantity },
}
