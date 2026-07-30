CREATE TABLE telemetry_metric (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    instance_uuid UUID NOT NULL,

    metric_id INT NOT NULL,
    namespace_id INT NOT NULL,
    name TEXT NULL,
    unit TEXT NULL,
    value BIGINT NOT NULL,

    payload BIGINT NULL,
    payload_info TEXT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
