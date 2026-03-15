CREATE TABLE title_stats (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    title_id uuid NOT NULL,
    bookmarks_count bigint NOT NULL DEFAULT 0,
    likes_count bigint NOT NULL DEFAULT 0,
    watch_count bigint NOT NULL DEFAULT 0,
    popularity bigint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_title_stats PRIMARY KEY (id),
    CONSTRAINT fkey_title_stats_to_titles FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_title_stats_on_title_id ON title_stats USING btree (title_id);
CREATE INDEX index_title_stats_on_popularity ON title_stats USING btree (popularity);

SELECT manage_updated_at('title_stats');
SELECT manage_versions('title_stats');

DROP INDEX index_titles_on_created_at;
