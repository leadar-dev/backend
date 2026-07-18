CREATE TABLE feature_flags (
    name       varchar(100) PRIMARY KEY,
    enabled    boolean      NOT NULL DEFAULT false,
    created_at timestamptz  NOT NULL DEFAULT now(),
    updated_at timestamptz  NOT NULL DEFAULT now()
);

CREATE TRIGGER trg_feature_flags_updated_at
    BEFORE UPDATE ON feature_flags
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

INSERT INTO feature_flags (name) VALUES
    ('ai'),
    ('finance'),
    ('elastic_search'),
    ('saved_filters');
