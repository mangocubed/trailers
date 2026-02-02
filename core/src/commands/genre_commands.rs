use uuid::Uuid;

use crate::db_pool;
use crate::models::{Genre, Title};
use crate::pagination::{CursorPage, CursorParams};

pub async fn get_genre_by_id<'a>(id: Uuid) -> sqlx::Result<Genre<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Genre,
        "SELECT id, tmdb_id, name, created_at, updated_at FROM genres WHERE id = $1 LIMIT 1",
        id // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_genre<'a>(tmdb_id: i32, name: &'a str) -> sqlx::Result<Genre<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Genre,
        "INSERT INTO genres (tmdb_id, name) VALUES ($1, $2) ON CONFLICT (tmdb_id) DO NOTHING RETURNING *",
        tmdb_id, // $1
        name,    // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_title_genre(title: &Title<'_>, genre: &Genre<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "INSERT INTO title_genres (title_id, genre_id) VALUES ($1, $2)",
        title.id, // $1
        genre.id  // $2
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn paginate_genres<'a>(cursor_params: &CursorParams, title: Option<&Title<'_>>) -> CursorPage<Genre<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        cursor_params,
        |node: &Genre| node.id,
        async |after| get_genre_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_name) = cursor_resource.map(|r| (Some(r.id), Some(r.name))).unwrap_or_default();
            let title_id = title.map(|t| t.id);

            sqlx::query_as!(
                Genre,
                "SELECT * FROM genres AS g
                WHERE
                    ($1::uuid IS NULL OR $2::text IS NULL OR name > $2 OR (name = $2 AND id > $1))
                    AND ($3::uuid IS NULL OR (
                        SELECT id FROM title_genres AS tg WHERE tg.title_id = $3 AND tg.genre_id = g.id LIMIT 1
                    ) IS NOT NULL)
                ORDER BY name ASC, id ASC LIMIT $4",
                cursor_id,              // $1
                cursor_name.as_deref(), // $2
                title_id,               // $3
                limit                   // $4
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}
