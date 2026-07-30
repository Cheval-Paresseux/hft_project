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
