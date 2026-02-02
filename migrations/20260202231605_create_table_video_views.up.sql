CREATE TABLE video_views (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    video_id uuid NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_video_views PRIMARY KEY (id),
    CONSTRAINT fkey_video_views_to_videos FOREIGN KEY (video_id) REFERENCES videos (id) ON DELETE CASCADE,
    CONSTRAINT fkey_video_views_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_video_views_on_video_id_user_id ON video_views USING btree (video_id, user_id);

SELECT manage_updated_at('video_views');
SELECT manage_versions('video_views');
