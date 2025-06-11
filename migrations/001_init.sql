CREATE SCHEMA IF NOT EXISTS cohort_early_warning;

CREATE TABLE IF NOT EXISTS cohort_early_warning.scholars (
    id UUID PRIMARY KEY,
    full_name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    cohort TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cohort_early_warning.signals (
    id UUID PRIMARY KEY,
    scholar_id UUID NOT NULL REFERENCES cohort_early_warning.scholars(id) ON DELETE CASCADE,
    signal_type TEXT NOT NULL,
    severity INT NOT NULL CHECK (severity BETWEEN 1 AND 5),
    note TEXT NOT NULL,
    occurred_at DATE NOT NULL,
    source_key TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cohort_early_warning_scholar ON cohort_early_warning.signals(scholar_id);
CREATE INDEX IF NOT EXISTS idx_cohort_early_warning_occurred ON cohort_early_warning.signals(occurred_at);
