CREATE TABLE categories (
    id          serial PRIMARY KEY,
    source      varchar(50)  NOT NULL,
    external_id integer      NOT NULL,
    name        varchar(255) NOT NULL,
    parent_id   integer      REFERENCES categories(id),
    created_at  timestamptz  NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_categories_source_external ON categories(source, external_id);
CREATE INDEX idx_categories_parent_id ON categories(parent_id);
