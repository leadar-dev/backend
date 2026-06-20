CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE wants (
    id                   bigserial    PRIMARY KEY,
    source               varchar(50)  NOT NULL,
    external_id          bigint       NOT NULL,
    name                 varchar(255) NOT NULL,
    description          text,
    price_limit          numeric(12, 2) NOT NULL,
    possible_price_limit numeric(12, 2) NOT NULL,
    category_id          integer      REFERENCES categories(id),
    max_days             integer,
    status               varchar(20)  NOT NULL DEFAULT 'active',
    kwork_count          integer,
    views                integer,
    hired_percent        numeric(5, 2),
    url                  varchar(512) NOT NULL,
    date_create          timestamptz  NOT NULL,
    date_expire          timestamptz,
    parsed_at            timestamptz,
    created_at           timestamptz  NOT NULL DEFAULT now(),
    updated_at           timestamptz  NOT NULL DEFAULT now(),
    CONSTRAINT chk_wants_status CHECK (status IN ('active', 'archive', 'closed')),
    CONSTRAINT chk_wants_source CHECK (source IN ('kwork', 'fl', 'upwork'))
);

CREATE UNIQUE INDEX idx_wants_source_external ON wants(source, external_id);

CREATE TRIGGER trg_wants_updated_at
    BEFORE UPDATE ON wants
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
