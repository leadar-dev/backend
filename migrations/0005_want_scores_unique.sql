ALTER TABLE want_scores
    ADD CONSTRAINT uq_want_scores_want_id UNIQUE (want_id);
