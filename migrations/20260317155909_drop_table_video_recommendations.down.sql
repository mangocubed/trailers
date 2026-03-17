CREATE TABLE video_recommendations (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    video_id uuid NOT NULL,
    user_id uuid NOT NULL,
    relevance smallint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_video_recommendations PRIMARY KEY (id)
);

CREATE TABLE video_views (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    video_id uuid NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_video_views PRIMARY KEY (id)
);
