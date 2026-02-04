use uuid::Uuid;

use crate::db_pool;
use crate::enums::TitleCrewJob;
use crate::models::{Person, Title, TitleCrew};
use crate::pagination::{CursorPage, CursorParams};

pub async fn get_title_crew_by_id<'a>(id: Uuid) -> sqlx::Result<TitleCrew<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        TitleCrew,
        r#"SELECT id, title_id, person_id, tmdb_credit_id, job as "job!: TitleCrewJob", created_at, updated_at
        FROM title_crew WHERE id = $1 LIMIT 1"#,
        id // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_title_crew(
    title: &Title<'_>,
    person: &Person<'_>,
    tmdb_credit_id: &str,
    job: TitleCrewJob,
) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "INSERT INTO title_crew (title_id, person_id, tmdb_credit_id, job) VALUES ($1, $2, $3, $4)",
        title.id,            // $1
        person.id,           // $2
        tmdb_credit_id,      // $3
        job as TitleCrewJob, // $4
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn paginate_title_crew<'a>(
    cursor_params: &CursorParams,
    title: &Title<'_>,
    jobs: Option<Vec<TitleCrewJob>>,
) -> CursorPage<TitleCrew<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        cursor_params,
        |node: &TitleCrew| node.id,
        async |after| get_title_crew_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_created_at) = cursor_resource
                .map(|r| (Some(r.id), Some(r.created_at)))
                .unwrap_or_default();

            sqlx::query_as!(
                TitleCrew,
                r#"SELECT id, title_id, person_id, tmdb_credit_id, job as "job!: TitleCrewJob", created_at, updated_at
                FROM title_crew
                WHERE
                    ($1::uuid IS NULL OR $2::timestamptz IS NULL OR created_at < $2 OR (created_at = $2 AND id < $1))
                    AND title_id = $3 AND $4::title_crew_job[] IS NULL OR job = ANY($4)
                ORDER BY created_at DESC, id DESC LIMIT $5"#,
                cursor_id,         // $1
                cursor_created_at, // $2
                title.id,          // $3
                jobs as _,         // $4
                limit              // $5
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}
