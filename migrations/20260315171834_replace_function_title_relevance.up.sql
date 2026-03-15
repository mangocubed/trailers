DROP FUNCTION get_title_relevance;

CREATE FUNCTION get_title_relevance(t_id uuid, u_id uuid) RETURNS bigint AS $$
DECLARE
    relevance bigint := 0;
    popularity bigint := 0;
BEGIN
    relevance := (
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

    IF relevance = 0 THEN RETURN 0; END IF;

    popularity := COALESCE((SELECT popularity FROM title_stats WHERE title_id = t_id LIMIT 1), 0);

    RETURN relevance + ROUND(popularity * (relevance / (relevance + popularity)));
END;
$$ LANGUAGE plpgsql;
