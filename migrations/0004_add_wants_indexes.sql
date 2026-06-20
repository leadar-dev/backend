-- users
CREATE UNIQUE INDEX idx_users_telegram_id ON users(telegram_id);

-- wants
CREATE INDEX idx_wants_source ON wants(source);
CREATE INDEX idx_wants_category_id ON wants(category_id);
CREATE INDEX idx_wants_status ON wants(status);
CREATE INDEX idx_wants_date_create ON wants(date_create DESC);
CREATE INDEX idx_wants_date_expire ON wants(date_expire);
CREATE INDEX idx_wants_source_category_status ON wants(source, category_id, status);
CREATE INDEX idx_wants_active ON wants(date_create DESC) WHERE status = 'active';

-- want_scores
CREATE INDEX idx_want_scores_want_id ON want_scores(want_id);
CREATE INDEX idx_want_scores_calculated_at ON want_scores(calculated_at DESC);
