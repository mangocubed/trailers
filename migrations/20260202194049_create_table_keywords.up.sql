CREATE TABLE keywords (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    tmdb_id integer NOT NULL,
    name varchar NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_keywords PRIMARY KEY (id)
);

CREATE UNIQUE INDEX index_keywords_on_tmdb_id ON keywords USING btree (tmdb_id);

SELECT manage_updated_at('keywords');
SELECT manage_versions('keywords');

CREATE TABLE title_keywords (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    title_id uuid NOT NULL,
    keyword_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_title_keywords PRIMARY KEY (id),
    CONSTRAINT fkey_title_keywords_to_titles FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
    CONSTRAINT fkey_title_keywords_to_keywords FOREIGN KEY (keyword_id) REFERENCES keywords (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_title_keywords_on_title_id_keyword_id ON title_keywords USING btree (title_id, keyword_id);

SELECT manage_updated_at('title_keywords');
SELECT manage_versions('title_keywords');
