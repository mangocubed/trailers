DROP INDEX index_titles_on_search;

CREATE INDEX index_titles_on_search ON titles USING gin (search);

CREATE INDEX index_titles_on_created_at ON titles USING btree (created_at);
