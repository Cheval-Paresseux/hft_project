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
    use super::*;

    #[test]
    fn inspect_ids_size() {
        println!("NamespaceId:  {} bytes", std::mem::size_of::<NamespaceId>());
        println!("ScopeId:  {} bytes", std::mem::size_of::<ScopeId>());
        println!("MetricId:  {} bytes", std::mem::size_of::<MetricId>());
    }

    #[test]
    fn ids_are_increasing() {
        let first = next_id();
        let second = next_id();

        assert!(second > first);
    }

    #[test]
    fn ids_are_unique() {
        let ids: Vec<_> = (0..100).map(|_| next_id()).collect();

        let mut sorted = ids.clone();
        sorted.sort_unstable();

        sorted.dedup();

        assert_eq!(sorted.len(), ids.len());
    }
}
