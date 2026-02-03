CREATE TABLE video_recommendations (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    video_id uuid NOT NULL,
    user_id uuid NOT NULL,
    relevance smallint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_video_recommendations PRIMARY KEY (id),
    CONSTRAINT fkey_video_recommendations_to_videos FOREIGN KEY (video_id) REFERENCES videos (id) ON DELETE CASCADE,
    CONSTRAINT fkey_video_recommendations_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_video_recommendations_on_video_id_user_id_ ON video_recommendations
USING btree (video_id, user_id);
CREATE INDEX index_video_recommendations_on_relevance ON video_recommendations USING btree (relevance);

SELECT manage_updated_at('video_recommendations');
SELECT manage_versions('video_recommendations');

CREATE FUNCTION get_title_relevance(t_id uuid, u_id uuid) RETURNS int AS $$
BEGIN
    RETURN (
        SELECT
            COALESCE(SUM((
                SELECT COUNT(utt.id) FROM user_title_ties AS utt, title_genres AS tg2
                WHERE
                    utt.user_id = u_id AND utt.title_id != t_id AND utt.title_id = tg2.title_id
                    AND tg2.genre_id = tg1.genre_id AND (utt.liked_at IS NOT NULL OR utt.bookmarked_at IS NOT NULL)
                LIMIT 1
            )), 0)
        FROM title_genres AS tg1 WHERE title_id = t_id
        LIMIT 1
    ) + (
        SELECT
            COALESCE(SUM((
                SELECT COUNT(utt.id) FROM user_title_ties AS utt, title_keywords AS tk2
                WHERE
                    utt.user_id = u_id AND utt.title_id != t_id AND utt.title_id = tk2.title_id
                    AND tk2.keyword_id = tk1.keyword_id AND (utt.liked_at IS NOT NULL OR utt.bookmarked_at IS NOT NULL)
                LIMIT 1
            )), 0)
        FROM title_keywords AS tk1 WHERE title_id = t_id
        LIMIT 1
    ) + (
        SELECT
            COALESCE(SUM((
                SELECT COUNT(utt.id) FROM user_title_ties AS utt, title_cast AS tc2
                WHERE
                    utt.user_id = u_id AND utt.title_id != t_id AND utt.title_id = tc2.title_id
                    AND tc2.person_id = tc1.person_id AND (utt.liked_at IS NOT NULL OR utt.bookmarked_at IS NOT NULL)
                LIMIT 1
            )), 0)
        FROM title_cast AS tc1 WHERE title_id = t_id
        LIMIT 1
    ) + (
        SELECT
            COALESCE(SUM((
                SELECT COUNT(utt.id) FROM user_title_ties AS utt, title_crew AS tc2
                WHERE
                    utt.user_id = u_id AND utt.title_id != t_id AND utt.title_id = tc2.title_id
                    AND tc2.person_id = tc1.person_id AND (utt.liked_at IS NOT NULL OR utt.bookmarked_at IS NOT NULL)
                LIMIT 1
            )), 0)
        FROM title_crew AS tc1 WHERE title_id = t_id
        LIMIT 1
    );
END;
$$ LANGUAGE plpgsql;
