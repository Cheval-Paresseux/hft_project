use super::namespace::TELEMETRY_NAMESPACE;

use crate::tracing::capture::ScopeId;
use crate::tracing::enrichment::ScopeMetadata;

// ── IDs Wrappers ──────────────────────────────────────────────────────────────

pub const INITIALIZATION: ScopeId = ScopeId::new(TELEMETRY_NAMESPACE, 0);
pub const SHUTDOWN: ScopeId = ScopeId::new(TELEMETRY_NAMESPACE, 1);

// ── Telemetry Scopes ──────────────────────────────────────────────────────────

pub static TELEMETRY_SCOPES: &[ScopeMetadata] = &[
    ScopeMetadata {
        id: INITIALIZATION,
        name: "initialization",
        description: "Initialization phase of the application or component.",
        payload_info: "N/A",
    },
    ScopeMetadata {
        id: SHUTDOWN,
        name: "shutdown",
        description: "Shutdown and cleanup phase of the application or component.",
        payload_info: "N/A",
    },
];

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn telemetry_scopes() {
        let mut ids = HashSet::new();
        let mut names = HashSet::new();

        for scope in TELEMETRY_SCOPES {
            assert!(ids.insert(scope.id), "duplicate scope id {:?}", scope.id);
            assert!(
                names.insert(scope.name),
                "duplicate scope name {}",
                scope.name
            );

            assert_eq!(scope.id.namespace, TELEMETRY_NAMESPACE);
            assert!(!scope.name.is_empty());
            assert!(!scope.description.is_empty());
            assert!(!scope.payload_info.is_empty());
        }

        assert_eq!(TELEMETRY_SCOPES.len(), 2);
    }
}
