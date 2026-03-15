use chrono::{NaiveDate, TimeDelta};
use sqlx::postgres::types::PgInterval;
use url::Url;
use uuid::Uuid;

use crate::db_pool;
use crate::enums::TitleMediaType;
use crate::models::{Title, User};
use crate::pagination::{CursorPage, CursorParams};

use super::{download_file, get_or_insert_title_stat};

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
    let query = query.unwrap_or_default().trim();

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
            released_on,
            CASE WHEN $2::uuid IS NOT NULL THEN
                COALESCE((SELECT relevance FROM title_recommendations WHERE title_id = $1 AND user_id = $2), 0) ELSE 0
            END AS "relevance!",
            (SELECT popularity FROM title_stats WHERE title_id = $1 LIMIT 1) AS "popularity!",
            CASE $3 WHEN '' THEN 0 ELSE COALESCE(ts_rank(search, websearch_to_tsquery($3)), 0) END AS "search_rank!",
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
            released_on,
            0 AS "relevance!",
            COALESCE((SELECT popularity FROM title_stats WHERE title_id = t.id LIMIT 1), 0) AS "popularity!",
            0::float4 AS "search_rank!",
            created_at,
            updated_at
        FROM titles AS t WHERE media_type = $1 AND tmdb_id = $2 LIMIT 1"#,
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
            released_on,
            0 AS "relevance!",
            0 AS "popularity!",
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

    let _ = get_or_insert_title_stat(&title).await;

    Ok(title)
}

pub async fn paginate_titles<'a>(
    cursor_params: CursorParams,
    user: Option<&User>,
    query: Option<&str>,
    include_viewed: Option<bool>,
) -> CursorPage<Title<'a>> {
    let db_pool = db_pool().await;
    let query = query.unwrap_or_default().trim();
    let include_viewed = include_viewed.unwrap_or_default();

    CursorPage::new(
        &cursor_params,
        |node: &Title| node.id,
        async |after| get_title_by_id(after, user, Some(query)).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_relevance, cursor_popularity, cursor_search_rank) = cursor_resource
                .map(|c| (Some(c.id), Some(c.relevance), Some(c.popularity), Some(c.search_rank)))
                .unwrap_or_default();
            let user_id = user.map(|u| u.id);

            sqlx::query_as!(
                Title,
                r#"SELECT
                    id as "id!",
                    media_type as "media_type!: TitleMediaType",
                    tmdb_id as "tmdb_id!",
                    tmdb_backdrop_path,
                    tmdb_poster_path,
                    imdb_id,
                    name as "name!",
                    overview as "overview!",
                    language as "language!",
                    runtime,
                    released_on,
                    relevance as "relevance!",
                    popularity as "popularity!",
                    search_rank as "search_rank!",
                    created_at as "created_at!",
                    updated_at
                FROM (
                    (SELECT
                        t.id,
                        media_type,
                        tmdb_id,
                        tmdb_backdrop_path,
                        tmdb_poster_path,
                        imdb_id,
                        name,
                        overview,
                        language,
                        runtime,
                        released_on,
                        tr.relevance,
                        COALESCE(ts.popularity, 0) AS popularity,
                        CASE $6 WHEN '' THEN 0 ELSE ts_rank(search, websearch_to_tsquery($6)) END AS search_rank,
                        t.created_at,
                        t.updated_at
                    FROM titles AS t
                    JOIN title_recommendations AS tr ON tr.user_id = $5 AND t.id = tr.title_id
                    LEFT JOIN title_stats AS ts ON ts.title_id = t.id
                    WHERE (
                            $1::uuid IS NULL
                            OR (tr.relevance, ts_rank(search, websearch_to_tsquery($6)), t.id) < ($2, $4, $1)
                        ) AND (
                            $6 = '' OR search @@ websearch_to_tsquery($6) OR name ILIKE '%'||$6||'%'
                            OR overview ILIKE '%'||$6||'%'
                        ) AND (
                            $7 IS TRUE
                            OR (SELECT id FROM user_title_ties WHERE title_id = t.id AND user_id = $5 LIMIT 1) IS NULL
                        ) AND (
                            tr.id IS NOT NULL
                            OR (
                                SELECT id FROM videos AS v WHERE title_id = t.id AND downloaded_at IS NOT NULL LIMIT 1
                            ) IS NOT NULL
                        )
                    ORDER BY
                        tr.relevance DESC,
                        CASE $6 WHEN '' THEN NULL ELSE ts_rank(search, websearch_to_tsquery($6)) END DESC,
                        t.id DESC
                    LIMIT $8)
                    UNION ALL
                    (SELECT
                        t.id,
                        media_type,
                        tmdb_id,
                        tmdb_backdrop_path,
                        tmdb_poster_path,
                        imdb_id,
                        name,
                        overview,
                        language,
                        runtime,
                        released_on,
                        0 AS relevance,
                        COALESCE(ts.popularity, 0) AS popularity,
                        CASE $6 WHEN '' THEN 0 ELSE ts_rank(search, websearch_to_tsquery($6)) END AS search_rank,
                        t.created_at,
                        t.updated_at
                    FROM titles AS t LEFT JOIN title_stats AS ts ON ts.title_id = t.id
                    WHERE (
                            $1::uuid IS NULL
                            OR (
                                COALESCE(ts.popularity, 0), ts_rank(search, websearch_to_tsquery($6)), t.id
                            ) < ($3, $4, $1)
                        ) AND (
                            $6 = '' OR search @@ websearch_to_tsquery($6) OR name ILIKE '%'||$6||'%'
                            OR overview ILIKE '%'||$6||'%'
                        ) AND (
                            $5 IS NULL OR (
                                SELECT id FROM title_recommendations WHERE user_id = $5 AND title_id = t.id LIMIT 1
                            ) IS NULL
                        ) AND (
                            $7 IS TRUE OR $5 IS NULL
                            OR (SELECT id FROM user_title_ties WHERE title_id = t.id AND user_id = $5 LIMIT 1) IS NULL
                        ) AND (
                            SELECT id FROM videos AS v WHERE title_id = t.id AND downloaded_at IS NOT NULL LIMIT 1
                        ) IS NOT NULL
                    ORDER BY
                        popularity DESC,
                        CASE $6 WHEN '' THEN NULL ELSE ts_rank(search, websearch_to_tsquery($6)) END DESC,
                        id DESC
                    LIMIT $8)
                ) LIMIT $8"#,
                cursor_id,          // $1
                cursor_relevance,   // $2
                cursor_popularity,  // $3
                cursor_search_rank, // $4
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
