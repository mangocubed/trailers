CREATE TABLE watch_providers (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    tmdb_id integer NOT NULL,
    tmdb_logo_path varchar NULL,
    name varchar NOT NULL,
    home_url varchar NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_watch_providers PRIMARY KEY (id)
);

CREATE UNIQUE INDEX index_watch_providers_on_tmdb_id ON watch_providers USING btree (tmdb_id);

SELECT manage_updated_at('watch_providers');
SELECT manage_versions('watch_providers');

CREATE TABLE title_watch_providers (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    title_id uuid NOT NULL,
    watch_provider_id uuid NOT NULL,
    country_codes varchar [] NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_title_watch_providers PRIMARY KEY (id),
    CONSTRAINT fkey_title_watch_providers_to_titles FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
    CONSTRAINT fkey_title_watch_providers_to_watch_providers FOREIGN KEY (watch_provider_id)
    REFERENCES watch_providers (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_title_watch_providers_on_title_id_watch_provider_id ON title_watch_providers
USING btree (title_id, watch_provider_id);

SELECT manage_updated_at('title_watch_providers');
SELECT manage_versions('title_watch_providers');
