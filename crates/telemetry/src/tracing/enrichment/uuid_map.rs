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
    use super::UUIDMap;

    #[test]
    fn get_or_create_scope_uuid() {
        let mut uuid_map = UUIDMap::default();

        let first = uuid_map.get_or_create_scope_uuid(0);
        let second = uuid_map.get_or_create_scope_uuid(0);
        assert_eq!(first, second);

        let other = uuid_map.get_or_create_scope_uuid(1);
        assert_ne!(first, other);

        assert_eq!(uuid_map.get_or_create_scope_uuid(1), other);
        assert_eq!(uuid_map.get_or_create_scope_uuid(0), first);
    }
}
