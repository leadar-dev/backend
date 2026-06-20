CREATE TABLE want_scores (
    id            bigserial    PRIMARY KEY,
    want_id       bigint       NOT NULL REFERENCES wants(id),
    zscore_price  numeric(10, 4),
    zscore_activity numeric(10, 4),
    trend_slope   numeric(10, 4),
    calculated_at timestamptz  NOT NULL DEFAULT now()
);

CREATE TABLE users (
    id          bigserial   PRIMARY KEY,
    telegram_id bigint      NOT NULL,
    is_active   boolean     NOT NULL DEFAULT true,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT uq_users_telegram_id UNIQUE (telegram_id)
);

CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
