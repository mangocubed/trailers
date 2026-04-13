ALTER TABLE titles DROP COLUMN search,
ADD COLUMN search tsvector GENERATED ALWAYS AS (to_tsvector('english', name || ' ' || overview)) STORED;
