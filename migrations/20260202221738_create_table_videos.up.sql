CREATE TYPE video_orientation AS ENUM ('portrait', 'landscape');

CREATE TYPE video_source AS ENUM ('youtube');

CREATE TYPE video_type AS ENUM ('teaser', 'trailer');

CREATE TABLE videos (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    title_id uuid NOT NULL,
    tmdb_id varchar NOT NULL,
    source video_source NOT NULL,
    source_key varchar NOT NULL,
    name varchar NOT NULL,
    video_type video_type NOT NULL,
    duration_secs integer NOT NULL DEFAULT 0,
    orientation video_orientation NOT NULL DEFAULT 'landscape',
    language varchar NOT NULL,
    published_at timestamptz NOT NULL DEFAULT current_timestamp,
    downloaded_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_videos PRIMARY KEY (id),
    CONSTRAINT fkey_videos_to_titles FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_videos_on_tmdb_id ON videos USING btree (tmdb_id);
CREATE UNIQUE INDEX index_videos_on_source_source_key ON videos USING btree (source, source_key);

SELECT manage_updated_at('videos');
SELECT manage_versions('videos');
