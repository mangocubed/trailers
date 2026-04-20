use uuid::Uuid;

use toolbox::pagination::{CursorPage, CursorParams};

use crate::db_pool;
use crate::models::{Genre, Title};

pub async fn get_genre_by_id<'a>(id: Uuid) -> sqlx::Result<Genre<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Genre,
        "SELECT * FROM genres WHERE id = $1 LIMIT 1",
        id // $1
    )
    .fetch_one(db_pool)
    .await
}

async fn get_genre_by_tmdb_id<'a>(tmdb_id: i32) -> sqlx::Result<Genre<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Genre,
        "SELECT * FROM genres WHERE tmdb_id = $1 LIMIT 1",
        tmdb_id // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_or_insert_genre<'a>(tmdb_id: i32, name: &'a str) -> sqlx::Result<Genre<'a>> {
    if let Ok(genre) = get_genre_by_tmdb_id(tmdb_id).await {
        return Ok(genre);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        Genre,
        "INSERT INTO genres (tmdb_id, name) VALUES ($1, $2) RETURNING *",
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

pub async fn paginate_genres<'a>(
    cursor_params: &CursorParams,
    ids: Option<Vec<Uuid>>,
    title: Option<&Title<'_>>,
) -> CursorPage<Genre<'a>> {
    let db_pool = db_pool().await;
    let ids = ids.unwrap_or_default();

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
                    ($1::uuid IS NULL OR (name, id) > ($2, $1))
                    AND (cardinality($3::uuid[]) = 0 OR id = ANY($3))
                    AND ($4::uuid IS NULL OR (
                        SELECT id FROM title_genres AS tg WHERE tg.title_id = $4 AND tg.genre_id = g.id LIMIT 1
                    ) IS NOT NULL)
                ORDER BY name ASC, id ASC LIMIT $5",
                cursor_id,              // $1
                cursor_name.as_deref(), // $2
                &ids,                   // $3
                title_id,               // $4
                limit                   // $5
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}
