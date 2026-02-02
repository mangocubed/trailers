use chrono::{NaiveDate, TimeDelta};
use sqlx::postgres::types::PgInterval;
use url::Url;
use uuid::Uuid;

use crate::db_pool;
use crate::enums::TitleMediaType;
use crate::models::Title;
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

pub async fn get_title_by_id(id: Uuid, query: Option<&str>) -> sqlx::Result<Title<'_>> {
    let db_pool = db_pool().await;

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
            created_at,
            updated_at,
            CASE WHEN $2::varchar IS NOT NULL THEN ts_rank(search, websearch_to_tsquery($2)) ELSE 0 END AS search_rank
        FROM titles WHERE id = $1 LIMIT 1"#,
        id,    // $1
        query, // $2
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
            created_at,
            updated_at,
            NULL::float4 AS search_rank
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
            created_at,
            updated_at,
            NULL::float4 AS search_rank"#,
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
        && let Some(dest_url) = title.backdrop_path()
    {
        let _ = download_file(source_url, dest_url).await;
    }

    if let Some(source_url) = tmdb_poster_url
        && let Some(dest_url) = title.poster_path()
    {
        let _ = download_file(source_url, dest_url).await;
    }

    Ok(title)
}

pub async fn paginate_titles<'a>(query: Option<String>, cursor_params: &CursorParams) -> CursorPage<Title<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        cursor_params,
        |node: &Title| node.id,
        async |after| get_title_by_id(after, None).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_search_rank) =
                cursor_resource.map(|c| (Some(c.id), c.search_rank)).unwrap_or_default();

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
                    created_at,
                    updated_at,
                    CASE WHEN $3::varchar IS NOT NULL THEN
                        ts_rank(search, websearch_to_tsquery($3))
                    ELSE
                        NULL
                    END AS search_rank
                FROM titles
                WHERE (
                        $1::uuid IS NULL OR $2::float4 IS NULL OR $3 IS NULL
                        OR ts_rank(search, websearch_to_tsquery($3)) < $2
                        OR (ts_rank(search, websearch_to_tsquery($3)) = $2 AND id < $1)
                    ) AND (
                        $3 IS NULL OR search @@ websearch_to_tsquery($3) OR name ILIKE '%'||$3||'%'
                        OR overview ILIKE '%'||$3||'%'
                    )
                ORDER BY search_rank DESC, id DESC LIMIT $4"#,
                cursor_id,          // $1
                cursor_search_rank, // $2
                query,              // $3
                limit,              // $4
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}
