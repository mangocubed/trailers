use uuid::Uuid;

use toolbox::pagination::{CursorPage, CursorParams};

use crate::db_pool;
use crate::models::{Keyword, Title};

pub async fn get_keyword_by_id<'a>(id: Uuid) -> sqlx::Result<Keyword<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Keyword,
        "SELECT * FROM keywords WHERE id = $1 LIMIT 1",
        id // $1
    )
    .fetch_one(db_pool)
    .await
}

async fn get_keyword_by_tmdb_id<'a>(tmdb_id: i32) -> sqlx::Result<Keyword<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Keyword,
        "SELECT * FROM keywords WHERE tmdb_id = $1 LIMIT 1",
        tmdb_id // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_or_insert_keyword(tmdb_id: i32, name: &str) -> sqlx::Result<Keyword<'_>> {
    if let Ok(keyword) = get_keyword_by_tmdb_id(tmdb_id).await {
        return Ok(keyword);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        Keyword,
        "INSERT INTO keywords (tmdb_id, name) VALUES ($1, $2) RETURNING *",
        tmdb_id, // $1
        name,    // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_title_keyword(title: &Title<'_>, keyword: &Keyword<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "INSERT INTO title_keywords (title_id, keyword_id) VALUES ($1, $2)",
        title.id,   // $1
        keyword.id  // $2
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn paginate_keywords<'a>(cursor_params: &CursorParams, title: Option<&Title<'_>>) -> CursorPage<Keyword<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        cursor_params,
        |node: &Keyword| node.id,
        async |after| get_keyword_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_name) = cursor_resource.map(|r| (Some(r.id), Some(r.name))).unwrap_or_default();
            let title_id = title.map(|t| t.id);

            sqlx::query_as!(
                Keyword,
                "SELECT * FROM keywords AS k
                WHERE
                    ($1::uuid IS NULL OR $2::text IS NULL OR name > $2 OR (name = $2 AND id > $1))
                    AND ($3::uuid IS NULL OR (
                        SELECT id FROM title_keywords AS tk WHERE tk.title_id = $3 AND tk.keyword_id = k.id LIMIT 1
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
