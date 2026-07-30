use crate::tracing::capture::ScopeInstanceId;

use std::collections::HashMap;
use uuid::Uuid;

// ── Uuid Map ──────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct UUIDMap {
    scope_ids_map: HashMap<ScopeInstanceId, Uuid>,
}

impl UUIDMap {
    pub fn get_or_create_scope_uuid(&mut self, instance_id: ScopeInstanceId) -> Uuid {
        *self
            .scope_ids_map
            .entry(instance_id)
            .or_insert_with(Uuid::new_v4)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_or_create_scope_uuid() {
        let mut uuid_map = UUIDMap::default();

        let uuid0 = uuid_map.get_or_create_scope_uuid(0);

        assert!(uuid0 == uuid_map.get_or_create_scope_uuid(0))
    }
}
