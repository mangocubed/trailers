use crate::db_pool;
use crate::models::User;

pub async fn update_title_recommendations(user: &User) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    let skip_update = sqlx::query!(
        "SELECT id FROM title_recommendations
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
        r#"DELETE FROM title_recommendations AS tr
        WHERE
            user_id = $1
            AND (SELECT id FROM user_title_ties WHERE user_id = $1 AND title_id = tr.title_id LIMIT 1) IS NOT NULL"#,
        user.id
    )
    .execute(db_pool)
    .await?;

    sqlx::query!(
        r#"INSERT INTO title_recommendations (user_id, title_id, relevance)
        SELECT $1 AS user_id, id AS title_id, get_title_relevance(id, $1) AS relevance
        FROM titles AS t
        WHERE is_adult IS FALSE
            AND (SELECT id FROM user_title_ties WHERE title_id = t.id AND user_id = $1 LIMIT 1) IS NULL
            AND (SELECT id FROM videos AS v WHERE title_id = t.id AND downloaded_at IS NOT NULL LIMIT 1) IS NOT NULL
            AND get_title_relevance(id, $1) > 0
        ON CONFLICT (title_id, user_id)
        DO UPDATE SET relevance=excluded.relevance"#,
        user.id,
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}
