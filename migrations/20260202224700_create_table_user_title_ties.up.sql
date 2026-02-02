CREATE TABLE user_title_ties (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    title_id uuid NOT NULL,
    bookmarked_at timestamptz NULL,
    bookmarked_video_id uuid NULL,
    liked_at timestamptz NULL,
    liked_video_id uuid NULL,
    watched_at timestamptz NULL,
    watched_video_id uuid NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_user_title_ties PRIMARY KEY (id),
    CONSTRAINT fkey_user_title_ties_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT fkey_user_title_ties_to_titles FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
    CONSTRAINT fkey_user_title_ties_to_bookmarked_videos FOREIGN KEY (bookmarked_video_id) REFERENCES videos (id)
    ON DELETE SET NULL,
    CONSTRAINT fkey_user_title_ties_to_liked_videos FOREIGN KEY (liked_video_id) REFERENCES videos (id)
    ON DELETE SET NULL,
    CONSTRAINT fkey_user_title_ties_to_watched_videos FOREIGN KEY (watched_video_id) REFERENCES videos (id)
    ON DELETE SET NULL
);

CREATE UNIQUE INDEX index_user_title_ties_on_user_id_title_id ON user_title_ties USING btree (user_id, title_id);

SELECT manage_updated_at('user_title_ties');
SELECT manage_versions('user_title_ties');
