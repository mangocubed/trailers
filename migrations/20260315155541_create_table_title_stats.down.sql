DROP TABLE title_stats;

CREATE INDEX index_titles_on_created_at ON titles USING btree (created_at);
