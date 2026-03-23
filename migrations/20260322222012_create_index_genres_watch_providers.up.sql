CREATE INDEX index_genres_on_name ON genres USING btree (name);

CREATE INDEX index_watch_providers_on_name ON watch_providers USING btree (name);

CREATE INDEX index_title_watch_providers_on_country_codes ON title_watch_providers USING gin (country_codes);
