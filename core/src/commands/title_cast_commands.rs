use uuid::Uuid;

use toolbox::pagination::{CursorPage, CursorParams};

use crate::db_pool;
use crate::models::{Person, Title, TitleCast};

pub async fn get_title_cast_by_id<'a>(id: Uuid) -> sqlx::Result<TitleCast<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        TitleCast,
        "SELECT * FROM title_cast WHERE id = $1 LIMIT 1",
        id // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_or_update_title_cast(
    title: &Title<'_>,
    person: &Person<'_>,
    tmdb_credit_id: &str,
    character_name: &str,
    position: i16,
) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        r#"INSERT INTO title_cast (title_id, person_id, tmdb_credit_id, character_name, position)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (title_id, person_id) DO UPDATE SET tmdb_credit_id = $3, character_name = $4, position = $5"#,
        title.id,       // $1
        person.id,      // $2
        tmdb_credit_id, // $3
        character_name, // $4
        position,       // $5
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn paginate_title_cast<'a>(cursor_params: &CursorParams, title: &Title<'_>) -> CursorPage<TitleCast<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        cursor_params,
        |node: &TitleCast| node.id,
        async |after| get_title_cast_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_position) = cursor_resource
                .map(|r| (Some(r.id), Some(r.position)))
                .unwrap_or_default();

            sqlx::query_as!(
                TitleCast,
                "SELECT * FROM title_cast WHERE title_id = $1 AND ($2::uuid IS NULL OR (position, id) > ($3, $2))
                ORDER BY position ASC, id ASC LIMIT $4",
                title.id,        // $1
                cursor_id,       // $2
                cursor_position, // $3
                limit            // $4
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}
