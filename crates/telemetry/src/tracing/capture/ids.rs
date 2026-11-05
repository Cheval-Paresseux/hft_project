use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};

// ── Instance ID  ──────────────────────────────────────────────────────────────

pub type ScopeInstanceId = u32;

static SCOPE_INSTANCE_ID: AtomicU32 = AtomicU32::new(0);

#[inline(always)]
pub fn next_id() -> ScopeInstanceId {
    SCOPE_INSTANCE_ID.fetch_add(1, Ordering::Relaxed)
}

// ── Namespace Id ──────────────────────────────────────────────────────────────

pub type NamespaceId = u16;

// ── Scope Id ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct ScopeId {
    pub namespace: u16,
    pub id: u16,
}

impl ScopeId {
    pub const fn new(namespace: NamespaceId, id: u16) -> Self {
        Self { namespace, id }
    }
}

// ── Metric Id ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct MetricId {
    pub namespace: u16,
    pub id: u16,
}

impl MetricId {
    pub const fn new(namespace: NamespaceId, id: u16) -> Self {
        Self { namespace, id }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::{MetricId, NamespaceId, ScopeId, ScopeInstanceId};
    use std::collections::HashSet;

    mod scope_id {
        use super::super::{NamespaceId, ScopeId};

        #[test]
        fn new() {
            let namespace: NamespaceId = 0x1201;

            let id = ScopeId::new(namespace, 42);

            assert_eq!(id.namespace, namespace);
            assert_eq!(id.id, 42);
        }
    }

    mod metric_id {
        use super::super::{MetricId, NamespaceId};

        #[test]
        fn new() {
            let namespace: NamespaceId = 0x1201;

            let id = MetricId::new(namespace, 42);

            assert_eq!(id.namespace, namespace);
            assert_eq!(id.id, 42);
        }
    }

    #[test]
    fn next_id() {
        let first = super::next_id();
        let second = super::next_id();

        assert!(second > first);

        let ids: Vec<_> = (0..100).map(|_| super::next_id()).collect();
        let unique: HashSet<_> = ids.iter().copied().collect();

        assert_eq!(unique.len(), ids.len());
    }

    #[test]
    fn ids_have_expected_layout() {
        assert_eq!(std::mem::size_of::<NamespaceId>(), 2);
        assert_eq!(std::mem::size_of::<ScopeInstanceId>(), 4);
        assert_eq!(std::mem::size_of::<ScopeId>(), 4);
        assert_eq!(std::mem::size_of::<MetricId>(), 4);
    }
}
