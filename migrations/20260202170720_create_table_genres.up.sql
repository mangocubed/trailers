CREATE TABLE genres (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    tmdb_id integer NOT NULL,
    name varchar NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_genres PRIMARY KEY (id)
);

CREATE UNIQUE INDEX index_genres_on_tmdb_id ON genres USING btree (tmdb_id);

SELECT manage_updated_at('genres');
SELECT manage_versions('genres');

CREATE TABLE title_genres (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    title_id uuid NOT NULL,
    genre_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_titles_genres PRIMARY KEY (id),
    CONSTRAINT fkey_title_genres_to_titles FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
    CONSTRAINT fkey_title_genres_to_genres FOREIGN KEY (genre_id) REFERENCES genres (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_title_genres_on_title_id_genre_id ON title_genres USING btree (title_id, genre_id);

SELECT manage_updated_at('title_genres');
SELECT manage_versions('title_genres');
