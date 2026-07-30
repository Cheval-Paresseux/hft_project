CREATE TABLE telemetry_scope (
    instance_uuid UUID PRIMARY KEY,
    parent_uuid UUID NULL,

    scope_id INT NOT NULL,
    namespace_id INT NOT NULL,
    scope_name TEXT NULL,

    payload BIGINT NULL,
    payload_info TEXT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
