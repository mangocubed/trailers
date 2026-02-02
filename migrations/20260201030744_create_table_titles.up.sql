CREATE TYPE title_media_type AS ENUM ('movie', 'series', 'short');

CREATE TABLE titles (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    media_type title_media_type NOT NULL,
    tmdb_id integer NOT NULL,
    tmdb_backdrop_path varchar NULL,
    tmdb_poster_path varchar NULL,
    imdb_id varchar NULL,
    name varchar NOT NULL,
    overview text NOT NULL,
    language varchar NOT NULL,
    runtime interval NULL,
    is_adult boolean NOT NULL DEFAULT FALSE,
    released_on date NULL,
    search tsvector GENERATED ALWAYS AS (to_tsvector('english', name || ' ' || overview)) STORED,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_titles PRIMARY KEY (id)
);

CREATE UNIQUE INDEX index_titles_on_media_type_tmdb_id ON titles USING btree (media_type, tmdb_id);
CREATE UNIQUE INDEX index_titles_on_imdb_id ON titles USING btree (imdb_id);

CREATE INDEX index_titles_on_search ON titles USING btree (search);

SELECT manage_updated_at('titles');
SELECT manage_versions('titles');
