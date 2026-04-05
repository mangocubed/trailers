use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use chrono::{NaiveDate, TimeDelta};
use sqlx::postgres::types::PgInterval;
use url::Url;
use uuid::Uuid;

use crate::constants::CACHE_PREFIX_GET_TITLE_BY_ID;
use crate::enums::TitleMediaType;
use crate::models::{Title, User};
use crate::pagination::{CursorPage, CursorParams};
use crate::{db_pool, jobs_storage};

use super::{AsyncRedisCacheExt, async_redis_cache, download_file, get_or_insert_title_stat};

pub async fn delete_title(title: &Title<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        r#"DELETE FROM titles WHERE id = $1"#,
        title.id // $1
    )
    .execute(db_pool)
    .await?;

    remove_title_cache(title).await;

    Ok(())
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Title<'_>>",
    create = r##"{ async_redis_cache(CACHE_PREFIX_GET_TITLE_BY_ID).await }"##
)]
async fn get_cached_title_by_id(id: Uuid) -> sqlx::Result<Title<'static>> {
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
            released_on,
            0 AS "relevance!",
            0 AS "popularity!",
            0::float4 AS "search_rank!",
            created_at,
            updated_at
        FROM titles WHERE id = $1 LIMIT 1"#,
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_title_by_id<'a>(id: Uuid, user: Option<&User>, query: Option<&str>) -> sqlx::Result<Title<'a>> {
    let user_id = user.map(|u| u.id);
    let query = query.unwrap_or_default().trim();

    if user_id.is_none() && query.is_empty() {
        return get_cached_title_by_id(id).await;
    };

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
            released_on,
            CASE WHEN $2::uuid IS NOT NULL THEN
                COALESCE((SELECT relevance FROM title_recommendations WHERE title_id = $1 AND user_id = $2), 0) ELSE 0
            END AS "relevance!",
            COALESCE((SELECT popularity FROM title_stats WHERE title_id = $1 LIMIT 1), 0) AS "popularity!",
            CASE $3 WHEN '' THEN 0 ELSE ts_rank(search, websearch_to_tsquery($3)) END AS "search_rank!",
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

    remove_title_cache(&title).await;

    Ok(title)
}

pub async fn paginate_titles<'a>(
    cursor_params: CursorParams,
    user: Option<&User>,
    query: Option<&str>,
    media_type: Option<TitleMediaType>,
    genre_ids: Option<Vec<Uuid>>,
    watch_provider_ids: Option<Vec<Uuid>>,
    country_code: Option<&str>,
    include_viewed: Option<bool>,
    include_without_videos: Option<bool>,
) -> CursorPage<Title<'a>> {
    let db_pool = db_pool().await;
    let query = query.unwrap_or_default().trim();
    let genre_ids = genre_ids.unwrap_or_default();
    let watch_provider_ids = watch_provider_ids.unwrap_or_default();
    let include_viewed = include_viewed.unwrap_or_default();
    let include_without_videos = include_without_videos.unwrap_or_default();

    CursorPage::new(
        &cursor_params,
        |node: &Title| node.id,
        async |after| get_title_by_id(after, user, Some(query)).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_relevance, cursor_popularity, cursor_search_rank) = cursor_resource
                .map(|c| (Some(c.id), Some(c.relevance), Some(c.popularity), Some(c.search_rank)))
                .unwrap_or_default();
            let user_id = user.map(|u| u.id);

            if !query.is_empty() && cursor_id.is_none() {
                jobs_storage().await.push_populate(Some(query.to_owned()), None, None).await;
            }

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
                        t.*,
                        tr.relevance,
                        COALESCE((SELECT popularity FROM title_stats WHERE title_id = t.id LIMIT 1), 0) AS popularity,
                        CASE $6 WHEN '' THEN 0 ELSE ts_rank(search, websearch_to_tsquery($6)) END AS search_rank
                    FROM titles AS t JOIN title_recommendations AS tr ON tr.user_id = $5 AND t.id = tr.title_id
                    WHERE (
                            $1::uuid IS NULL
                            OR (tr.relevance, ts_rank(search, websearch_to_tsquery($6)), t.id) < ($2, $4, $1)
                        ) AND ($6 = '' OR search @@ websearch_to_tsquery($6))
                        AND ($7::title_media_type IS NULL OR media_type = $7)
                        AND (
                            cardinality($8::uuid[]) = 0
                            OR (SELECT id FROM title_genres WHERE title_id = t.id AND genre_id = ANY($8) LIMIT 1) IS NOT NULL
                        ) AND (
                            cardinality($9::uuid[]) = 0
                            OR (
                                SELECT id FROM title_watch_providers
                                WHERE
                                    title_id = t.id AND watch_provider_id = ANY($9)
                                    AND ($10::text IS NULL OR $10 = ANY(country_codes))
                                LIMIT 1
                            ) IS NOT NULL
                        ) AND (
                            $11 IS TRUE
                            OR (SELECT id FROM user_title_ties WHERE title_id = t.id AND user_id = $5 LIMIT 1) IS NULL
                        ) AND ($12 IS TRUE OR has_videos IS TRUE)
                    ORDER BY
                        tr.relevance DESC,
                        CASE $6 WHEN '' THEN NULL ELSE ts_rank(search, websearch_to_tsquery($6)) END DESC,
                        t.id DESC
                    LIMIT $13)
                    UNION ALL
                    (SELECT
                        t.*,
                        0 AS relevance,
                        ts.popularity,
                        CASE $6 WHEN '' THEN 0 ELSE ts_rank(search, websearch_to_tsquery($6)) END AS search_rank
                    FROM titles AS t JOIN title_stats AS ts ON ts.title_id = t.id
                    WHERE (
                            $1::uuid IS NULL
                            OR (ts.popularity, ts_rank(search, websearch_to_tsquery($6)), t.id) < ($3, $4, $1)
                        ) AND (
                            $6 = '' OR search @@ websearch_to_tsquery($6) OR name ILIKE '%'||$6||'%'
                            OR overview ILIKE '%'||$6||'%'
                        ) AND ($7 IS NULL OR media_type = $7)
                        AND (
                            cardinality($8::uuid[]) = 0
                            OR (SELECT id FROM title_genres WHERE title_id = t.id AND genre_id = ANY($8) LIMIT 1) IS NOT NULL
                        ) AND (
                            cardinality($9::uuid[]) = 0
                            OR (
                                SELECT id FROM title_watch_providers
                                WHERE
                                    title_id = t.id AND watch_provider_id = ANY($9)
                                    AND ($10::text IS NULL OR $10 = ANY(country_codes))
                                LIMIT 1
                            ) IS NOT NULL
                        ) AND (
                            $5 IS NULL OR (
                                SELECT id FROM title_recommendations WHERE user_id = $5 AND title_id = t.id LIMIT 1
                            ) IS NULL
                        ) AND (
                            $11 IS TRUE OR $5 IS NULL
                            OR (SELECT id FROM user_title_ties WHERE title_id = t.id AND user_id = $5 LIMIT 1) IS NULL
                        ) AND ($12 IS TRUE OR has_videos IS TRUE)
                    ORDER BY
                        ts.popularity DESC,
                        CASE $6 WHEN '' THEN NULL ELSE ts_rank(search, websearch_to_tsquery($6)) END DESC,
                        id DESC
                    LIMIT $13)
                    UNION ALL
                    (SELECT
                        t.*,
                        0 AS relevance,
                        0 AS popularity,
                        CASE $6 WHEN '' THEN 0 ELSE ts_rank(search, websearch_to_tsquery($6)) END AS search_rank
                    FROM titles AS t
                    WHERE
                        ($1::uuid IS NULL OR (ts_rank(search, websearch_to_tsquery($6)), t.id) < ($4, $1))
                        AND (
                            $6 = '' OR search @@ websearch_to_tsquery($6) OR name ILIKE '%'||$6||'%'
                            OR overview ILIKE '%'||$6||'%'
                        ) AND ($7 IS NULL OR media_type = $7)
                        AND (
                            cardinality($8::uuid[]) = 0
                            OR (SELECT id FROM title_genres WHERE title_id = t.id AND genre_id = ANY($8) LIMIT 1) IS NOT NULL
                        ) AND (
                            cardinality($9::uuid[]) = 0
                            OR (
                                SELECT id FROM title_watch_providers
                                WHERE
                                    title_id = t.id AND watch_provider_id = ANY($9)
                                    AND ($10::text IS NULL OR $10 = ANY(country_codes))
                                LIMIT 1
                            ) IS NOT NULL
                        ) AND (
                            $5 IS NULL OR (
                                SELECT id FROM title_recommendations WHERE user_id = $5 AND title_id = t.id LIMIT 1
                            ) IS NULL
                        ) AND (
                            $11 IS TRUE OR $5 IS NULL
                            OR (SELECT id FROM user_title_ties WHERE title_id = t.id AND user_id = $5 LIMIT 1) IS NULL
                        ) AND ($12 IS TRUE OR has_videos IS TRUE)
                        AND (SELECT id FROM title_stats AS ts WHERE ts.title_id = t.id LIMIT 1) IS NULL
                    ORDER BY
                        CASE $6 WHEN '' THEN NULL ELSE ts_rank(search, websearch_to_tsquery($6)) END DESC,
                        id DESC
                    LIMIT $13)
                ) LIMIT $13"#,
                cursor_id,              // $1
                cursor_relevance,       // $2
                cursor_popularity,      // $3
                cursor_search_rank,     // $4
                user_id,                // $5
                query,                  // $6
                media_type as _,        // $7
                &genre_ids,             // $8
                &watch_provider_ids,    // $9
                country_code,           // $10
                include_viewed,         // $11
                include_without_videos, // $12
                limit,                  // $13
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}

pub async fn remove_title_cache(title: &Title<'_>) {
    GET_CACHED_TITLE_BY_ID
        .cache_remove(CACHE_PREFIX_GET_TITLE_BY_ID, &title.id)
        .await;
}
