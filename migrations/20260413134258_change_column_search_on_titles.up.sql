ALTER TABLE titles DROP COLUMN search,
ADD COLUMN search tsvector GENERATED ALWAYS AS
(setweight(to_tsvector('english', name), 'A') || setweight(to_tsvector('english', overview), 'B')) STORED;
