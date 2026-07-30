use super::namespace::TELEMETRY_NAMESPACE;

use crate::tracing::capture::MetricId;
use crate::tracing::enrichment::MetricMetadata;

// ── IDs Wrappers ──────────────────────────────────────────────────────────────

pub const SCOPE_START_TIME: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 0);
pub const SCOPE_END_TIME: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 1);

pub const START_TIME: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 10);
pub const END_TIME: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 11);

pub const EVENTS: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 20);
pub const ERRORS: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 21);
pub const WARNINGS: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 22);

pub const MEMORY_ALLOCATIONS: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 30);
pub const MEMORY_DEALLOCATIONS: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 31);
pub const MEMORY_REALLOCATIONS: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 32);
pub const MEMORY_ALLOCATED: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 33);
pub const MEMORY_DEALLOCATED: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 34);

pub const RESIDENT_MEMORY: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 40);
pub const RESIDENT_MEMORY_PEAK: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 41);
pub const VIRTUAL_MEMORY: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 42);
pub const VIRTUAL_MEMORY_PEAK: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 43);
pub const SWAP_MEMORY: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 44);
pub const LOCKED_MEMORY: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 45);

pub const PROCESS_USER_CPU_TIME: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 50);
pub const PROCESS_SYSTEM_CPU_TIME: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 51);

pub const ITEMS_PROCESSED: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 60);
pub const BYTES_PROCESSED: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 61);

pub const VALUE: MetricId = MetricId::new(TELEMETRY_NAMESPACE, 100);

// ── Telemetry Metrics ─────────────────────────────────────────────────────────

pub static TELEMETRY_METRICS: &[MetricMetadata] = &[
    MetricMetadata {
        id: SCOPE_START_TIME,
        name: "scope_start_time",
        unit: "ns",
        description: "Timestamp, in nanoseconds, when a scope started.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: SCOPE_END_TIME,
        name: "scope_end_time",
        unit: "ns",
        description: "Timestamp, in nanoseconds, when a scope ended.",
        payload_info: "N/A",
    },
    // ---
    MetricMetadata {
        id: START_TIME,
        name: "start_time",
        unit: "ns",
        description: "Timestamp, in nanoseconds, when an event started.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: END_TIME,
        name: "end_time",
        unit: "ns",
        description: "Timestamp, in nanoseconds, when an event ended.",
        payload_info: "N/A",
    },
    // ---
    MetricMetadata {
        id: EVENTS,
        name: "events",
        unit: "count",
        description: "Number of events observed.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: ERRORS,
        name: "errors",
        unit: "count",
        description: "Number of errors encountered.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: WARNINGS,
        name: "warnings",
        unit: "count",
        description: "Number of warnings emitted.",
        payload_info: "N/A",
    },
    // ---
    MetricMetadata {
        id: MEMORY_ALLOCATIONS,
        name: "memory_allocations",
        unit: "count",
        description: "Total number of allocations made.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: MEMORY_DEALLOCATIONS,
        name: "memory_deallocations",
        unit: "count",
        description: "Total number of deallocations made.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: MEMORY_REALLOCATIONS,
        name: "memory_reallocations",
        unit: "count",
        description: "Total number of reallocations made.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: MEMORY_ALLOCATED,
        name: "memory_allocated",
        unit: "bytes",
        description: "Total number of bytes allocated.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: MEMORY_DEALLOCATED,
        name: "memory_deallocated",
        unit: "bytes",
        description: "Total number of bytes deallocated.",
        payload_info: "N/A",
    },
    // ---
    MetricMetadata {
        id: RESIDENT_MEMORY,
        name: "memory_in_use",
        unit: "bytes",
        description: "Current resident memory used by the process (RSS: Resident Set Size).",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: RESIDENT_MEMORY_PEAK,
        name: "memory_peak",
        unit: "bytes",
        description: "Peak resident memory used by the process (HWM: High Water Mark).",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: VIRTUAL_MEMORY,
        name: "virtual_memory",
        unit: "bytes",
        description: "Current virtual memory allocated to the process.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: VIRTUAL_MEMORY_PEAK,
        name: "virtual_memory_peak",
        unit: "bytes",
        description: "Peak virtual memory allocated to the process.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: SWAP_MEMORY,
        name: "swap_memory",
        unit: "bytes",
        description: "Amount of swap memory currently used by the process.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: LOCKED_MEMORY,
        name: "locked_memory",
        unit: "bytes",
        description: "Amount of memory locked in RAM and prevented from being swapped.",
        payload_info: "N/A",
    },
    // ---
    MetricMetadata {
        id: PROCESS_USER_CPU_TIME,
        name: "process_user_cpu_time",
        unit: "ticks",
        description: "CPU time spent executing user-space code.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: PROCESS_SYSTEM_CPU_TIME,
        name: "process_system_cpu_time",
        unit: "ticks",
        description: "CPU time spent executing kernel-space code.",
        payload_info: "N/A",
    },
    // ---
    MetricMetadata {
        id: ITEMS_PROCESSED,
        name: "items_processed",
        unit: "count",
        description: "Number of items processed.",
        payload_info: "N/A",
    },
    MetricMetadata {
        id: BYTES_PROCESSED,
        name: "bytes_processed",
        unit: "bytes",
        description: "Total number of bytes processed.",
        payload_info: "N/A",
    },
    // ---
    MetricMetadata {
        id: VALUE,
        name: "value",
        unit: "",
        description: "Generic numeric value when no standard telemetry metric applies.",
        payload_info: "N/A",
    },
];
