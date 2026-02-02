CREATE TABLE persons (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    tmdb_id integer NOT NULL,
    tmdb_profile_path varchar NULL,
    imdb_id varchar NULL,
    name varchar NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_persons PRIMARY KEY (id)
);

CREATE UNIQUE INDEX index_persons_on_tmdb_id ON persons USING btree (tmdb_id);
CREATE UNIQUE INDEX index_persons_on_imdb_id ON persons USING btree (imdb_id);

SELECT manage_updated_at('persons');
SELECT manage_versions('persons');
