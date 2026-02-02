use uuid::Uuid;

use crate::db_pool;
use crate::models::{Person, Title, TitleCast};
use crate::pagination::{CursorPage, CursorParams};

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

pub async fn insert_title_cast(
    title: &Title<'_>,
    person: &Person<'_>,
    tmdb_credit_id: &str,
    character_name: &str,
) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        r#"INSERT INTO title_cast (title_id, person_id, tmdb_credit_id, character_name) VALUES ($1, $2, $3, $4)"#,
        title.id,       // $1
        person.id,      // $2
        tmdb_credit_id, // $3
        character_name, // $4
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn paginate_title_cast<'a>(
    cursor_params: &CursorParams,
    title: Option<&Title<'_>>,
) -> CursorPage<TitleCast<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        cursor_params,
        |node: &TitleCast| node.id,
        async |after| get_title_cast_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_character_name) = cursor_resource
                .map(|r| (Some(r.id), Some(r.character_name)))
                .unwrap_or_default();
            let title_id = title.map(|t| t.id);

            sqlx::query_as!(
                TitleCast,
                "SELECT * FROM title_cast
                WHERE
                    ($1::uuid IS NULL OR $2::text IS NULL OR character_name > $2 OR (character_name = $2 AND id > $1))
                    AND ($3::uuid IS NULL OR title_id = $3)
                ORDER BY character_name ASC, id ASC LIMIT $4",
                cursor_id,                        // $1
                cursor_character_name.as_deref(), // $2
                title_id,                         // $3
                limit                             // $4
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}
