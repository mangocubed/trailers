use crate::db_pool;
use crate::models::{Title, TitleStat};

pub async fn get_or_insert_title_stat(title: &Title<'_>) -> sqlx::Result<TitleStat> {
    let db_pool = db_pool().await;

    if let Ok(title_stat) = sqlx::query_as!(
        TitleStat,
        "SELECT id, title_id, created_at, updated_at FROM title_stats WHERE title_id = $1",
        title.id
    )
    .fetch_one(db_pool)
    .await
    {
        return Ok(title_stat);
    }

    sqlx::query_as!(
        TitleStat,
        "INSERT INTO title_stats (title_id) VALUES ($1) RETURNING id, title_id, created_at, updated_at",
        title.id
    )
    .fetch_one(db_pool)
    .await
}

pub async fn update_title_stat(title_stat: &TitleStat) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "UPDATE title_stats AS ts
        SET
            bookmarks_count = (
                SELECT COUNT(*) FROM user_title_ties AS utt WHERE utt.title_id = ts.title_id AND bookmarked_at IS NOT NULL
            ),
            likes_count = (
                SELECT COUNT(*) FROM user_title_ties AS utt WHERE utt.title_id = ts.title_id AND liked_at IS NOT NULL
            ),
            watch_count = (
                SELECT COUNT(*) FROM user_title_ties AS utt WHERE utt.title_id = ts.title_id AND watched_at IS NOT NULL
            ),
            popularity = (
                SELECT COUNT(*) FROM user_title_ties AS utt
                WHERE
                    utt.title_id = ts.title_id
                    AND (bookmarked_at IS NOT NULL OR liked_at IS NOT NULL OR watched_at IS NOT NULL)
            )
        WHERE id = $1",
        title_stat.id
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}
