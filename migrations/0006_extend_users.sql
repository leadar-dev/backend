ALTER TABLE users
    ADD COLUMN role        varchar(10)  NOT NULL DEFAULT 'user'
                               CONSTRAINT chk_users_role CHECK (role IN ('user', 'admin')),
    ADD COLUMN last_login  timestamptz,
    ADD COLUMN first_name  varchar(255),
    ADD COLUMN username    varchar(255);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
