use crate::db_pool;
use crate::models::User;

pub async fn update_video_recommendations(user: &User<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    let skip_update = sqlx::query!(
        "SELECT id FROM video_recommendations
            WHERE
                user_id = $1
                AND (
                    created_at > current_timestamp - INTERVAL '1 day' OR (
                        updated_at IS NOT NULL AND updated_at > current_timestamp - INTERVAL '1 day'
                    )
                )
            LIMIT 1",
        user.id,
    )
    .fetch_one(db_pool)
    .await
    .is_ok();

    if skip_update {
        return Ok(());
    }

    sqlx::query!(
        r#"DELETE FROM video_recommendations AS vr
            WHERE user_id = $1 AND (
                (
                    SELECT vv.id FROM video_views AS vv WHERE vv.user_id = $1 AND vv.video_id = vr.video_id
                    LIMIT 1
                ) IS NOT NULL OR (
                    SELECT v.id FROM videos AS v WHERE v.id = vr.video_id AND get_title_relevance(v.title_id, $1) = 0
                    LIMIT 1
                ) IS NOT NULL
            )"#,
        user.id
    )
    .execute(db_pool)
    .await?;

    sqlx::query!(
        r#"INSERT INTO video_recommendations (video_id, user_id, relevance)
            SELECT video_id, user_id, relevance FROM (
                SELECT
                    DISTINCT ON (title_id) title_id,
                    id AS video_id,
                    $1::uuid AS user_id,
                    (
                        SELECT COUNT(utt.id) FROM user_title_ties AS utt
                        WHERE
                            utt.user_id != $1 AND (utt.liked_at IS NOT NULL OR utt.bookmarked_at IS NOT NULL)
                            AND (utt.liked_video_id = v.id OR utt.bookmarked_video_id = v.id)
                        LIMIT 1
                    ) + get_title_relevance(title_id, $1) AS relevance
                FROM videos AS v
                WHERE
                    get_title_relevance(title_id, $1) > 0
                    AND (
                        SELECT vv.id FROM video_views AS vv WHERE vv.user_id = $1 AND vv.video_id = v.id
                        LIMIT 1
                    ) IS NULL
                    AND (SELECT is_adult FROM titles AS t WHERE t.id = v.title_id LIMIT 1) IS FALSE
                ORDER BY title_id DESC, relevance DESC, orientation::text DESC, published_at DESC
            ) AS sub
            ORDER BY relevance DESC
            ON CONFLICT (video_id, user_id)
            DO UPDATE SET relevance=excluded.relevance"#,
        user.id,
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}
