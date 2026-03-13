use chrono::{NaiveDate, TimeDelta};
use sqlx::postgres::types::PgInterval;
use url::Url;
use uuid::Uuid;

use crate::db_pool;
use crate::enums::TitleMediaType;
use crate::models::{Title, User};
use crate::pagination::{CursorPage, CursorParams};

use super::download_file;

pub async fn delete_title(title: &Title<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        r#"DELETE FROM titles WHERE id = $1"#,
        title.id // $1
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get_title_by_id<'a>(id: Uuid, user: Option<&User>, query: Option<&str>) -> sqlx::Result<Title<'a>> {
    let db_pool = db_pool().await;
    let user_id = user.map(|u| u.id);

    sqlx::query_as!(
        Title,
        r#"SELECT
            id,
            media_type AS "media_type!: TitleMediaType",
            tmdb_id,
            tmdb_backdrop_path,
            tmdb_poster_path,
            imdb_id,
            name,
            overview,
            language,
            runtime,
            is_adult,
            released_on,
            CASE WHEN $2::uuid IS NOT NULL THEN
                COALESCE((SELECT relevance FROM title_recommendations WHERE title_id = $1 AND user_id = $2), 0) ELSE 0
            END AS "relevance!",
            CASE WHEN $3::text IS NOT NULL THEN
                COALESCE(ts_rank(search, websearch_to_tsquery($3)), 0) ELSE 0
            END AS "search_rank!",
            created_at,
            updated_at
        FROM titles WHERE id = $1 LIMIT 1"#,
        id,      // $1
        user_id, // $2
        query,   // $3
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_title_by_tmdb_id<'a>(media_type: TitleMediaType, tmdb_id: i32) -> sqlx::Result<Title<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Title,
        r#"SELECT
            id,
            media_type as "media_type!: TitleMediaType",
            tmdb_id,
            tmdb_backdrop_path,
            tmdb_poster_path,
            imdb_id,
            name,
            overview,
            language,
            runtime,
            is_adult,
            released_on,
            0 AS "relevance!",
            0::float4 AS "search_rank!",
            created_at,
            updated_at
        FROM titles WHERE media_type = $1 AND tmdb_id = $2 LIMIT 1"#,
        media_type as TitleMediaType, // $1
        tmdb_id                       // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_or_update_title<'a>(
    media_type: TitleMediaType,
    tmdb_id: i32,
    tmdb_backdrop_path: Option<&'a str>,
    tmdb_backdrop_url: Option<Url>,
    tmdb_poster_path: Option<&'a str>,
    tmdb_poster_url: Option<Url>,
    imdb_id: Option<&'a str>,
    name: &'a str,
    overview: &'a str,
    language: &'a str,
    runtime: Option<TimeDelta>,
    is_adult: bool,
    released_on: Option<NaiveDate>,
) -> sqlx::Result<Title<'a>> {
    let db_pool = db_pool().await;

    let runtime = runtime.and_then(|r| PgInterval::try_from(r).ok());

    let title = sqlx::query_as!(
        Title,
        r#"INSERT INTO titles (
            media_type,
            tmdb_id,
            tmdb_backdrop_path,
            tmdb_poster_path,
            imdb_id,
            name,
            overview,
            language,
            runtime,
            is_adult,
            released_on
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (media_type, tmdb_id) DO UPDATE SET
            tmdb_backdrop_path = $3,
            tmdb_poster_path = $4,
            imdb_id = $5,
            name = $6,
            overview = $7,
            language = $8,
            runtime = $9,
            is_adult = $10,
            released_on = $11
        RETURNING
            id,
            media_type as "media_type!: TitleMediaType",
            tmdb_id,
            tmdb_backdrop_path,
            tmdb_poster_path,
            imdb_id,
            name,
            overview,
            language,
            runtime,
            is_adult,
            released_on,
            0 AS "relevance!",
            0::float4 AS "search_rank!",
            created_at,
            updated_at"#,
        media_type as _,    // $1
        tmdb_id,            // $2
        tmdb_backdrop_path, // $3
        tmdb_poster_path,   // $4
        imdb_id,            // $5
        name,               // $6
        overview,           // $7
        language,           // $8
        runtime,            // $9
        is_adult,           // $10
        released_on,        // $11
    )
    .fetch_one(db_pool)
    .await?;

    if let Some(source_url) = tmdb_backdrop_url
        && let Some(dest_url) = title.backdrop_image_path()
    {
        let _ = download_file(source_url, dest_url).await;
    }

    if let Some(source_url) = tmdb_poster_url
        && let Some(dest_url) = title.poster_image_path()
    {
        let _ = download_file(source_url, dest_url).await;
    }

    Ok(title)
}

pub async fn paginate_titles<'a>(
    cursor_params: CursorParams,
    user: Option<&User>,
    query: Option<String>,
    include_viewed: Option<bool>,
) -> CursorPage<Title<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        &cursor_params,
        |node: &Title| node.id,
        async |after| get_title_by_id(after, None, None).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_relevance, cursor_search_rank, cursor_created_at) =
                cursor_resource.map(|c| (Some(c.id), Some(c.relevance), Some(c.search_rank), Some(c.created_at))).unwrap_or_default();
            let user_id = user.map(|u| u.id);

            sqlx::query_as!(
                Title,
                r#"SELECT
                    t.id,
                    media_type as "media_type!: TitleMediaType",
                    tmdb_id,
                    tmdb_backdrop_path,
                    tmdb_poster_path,
                    imdb_id,
                    name,
                    overview,
                    language,
                    runtime,
                    is_adult,
                    released_on,
                    COALESCE(tr.relevance, 0) AS "relevance!",
                    CASE WHEN $6::text IS NOT NULL THEN
                        COALESCE(ts_rank(search, websearch_to_tsquery($6)), 0)
                    END AS "search_rank!",
                    t.created_at,
                    t.updated_at
                FROM titles AS t LEFT JOIN title_recommendations AS tr ON t.id = tr.title_id AND tr.user_id = $5
                WHERE (
                        $1::uuid IS NULL OR COALESCE(tr.relevance, 0) < $2
                        OR (COALESCE(tr.relevance, 0) = $2 AND COALESCE(ts_rank(search, websearch_to_tsquery($6)), 0) < $3)
                        OR (COALESCE(ts_rank(search, websearch_to_tsquery($6)), 0) = $3 AND t.created_at < $4)
                        OR (t.created_at = $4 AND t.id < $1)
                    ) AND (
                        $6 IS NULL OR search @@ websearch_to_tsquery($6) OR name ILIKE '%'||$6||'%'
                        OR overview ILIKE '%'||$6||'%'
                    ) AND (
                        $7 IS TRUE
                        OR (SELECT id FROM user_title_ties WHERE title_id = t.id AND user_id = $5 LIMIT 1) IS NULL
                    ) AND (tr.id IS NOT NULL OR (SELECT id FROM videos AS v WHERE title_id = t.id LIMIT 1) IS NOT NULL)
                ORDER BY "relevance!" DESC, "search_rank!" DESC, created_at DESC, id DESC LIMIT $8"#,
                cursor_id,          // $1
                cursor_relevance,   // $2
                cursor_search_rank, // $3
                cursor_created_at,  // $4
                user_id,            // $5
                query,              // $6
                include_viewed,     // $7
                limit,              // $8
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}
