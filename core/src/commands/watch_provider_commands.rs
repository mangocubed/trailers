use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use url::Url;
use uuid::Uuid;

use crate::constants::CACHE_PREFIX_GET_WATCH_PROVIDER_BY_ID;
use crate::db_pool;
use crate::models::{Title, TitleWatchProvider, WatchProvider};
use crate::pagination::{CursorPage, CursorParams};

use super::{async_redis_cache, download_file};

async fn get_title_watch_provider_by_id(id: Uuid) -> sqlx::Result<TitleWatchProvider> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        TitleWatchProvider,
        "SELECT * FROM title_watch_providers WHERE id = $1 LIMIT 1",
        id
    )
    .fetch_one(db_pool)
    .await
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, WatchProvider<'_>>",
    create = r##"{ async_redis_cache(CACHE_PREFIX_GET_WATCH_PROVIDER_BY_ID).await }"##
)]
pub async fn get_watch_provider_by_id(id: Uuid) -> sqlx::Result<WatchProvider<'static>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        WatchProvider,
        "SELECT * FROM watch_providers WHERE id = $1 LIMIT 1",
        id // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_watch_provider_by_tmdb_id<'a>(tmdb_id: i32) -> sqlx::Result<WatchProvider<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        WatchProvider,
        "SELECT * FROM watch_providers WHERE tmdb_id = $1 LIMIT 1",
        tmdb_id // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_or_insert_watch_provider<'a>(
    tmdb_id: i32,
    name: &'a str,
    tmdb_logo_path: Option<&'a str>,
    tmdb_logo_url: Option<Url>,
) -> sqlx::Result<WatchProvider<'a>> {
    if let Ok(watch_provider) = get_watch_provider_by_tmdb_id(tmdb_id).await {
        return Ok(watch_provider);
    }

    let db_pool = db_pool().await;

    let watch_provider = sqlx::query_as!(
        WatchProvider,
        "INSERT INTO watch_providers (tmdb_id, name, tmdb_logo_path) VALUES ($1, $2, $3) RETURNING *",
        tmdb_id,        // $1
        name,           // $2
        tmdb_logo_path  // $3
    )
    .fetch_one(db_pool)
    .await?;

    if let Some(source_url) = tmdb_logo_url
        && let Some(dest_url) = watch_provider.logo_image_path()
    {
        let _ = download_file(source_url, dest_url).await;
    }

    Ok(watch_provider)
}

pub async fn insert_or_update_title_watch_provider(
    title: &Title<'_>,
    watch_provider: &WatchProvider<'_>,
    country_codes: &[String],
) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "INSERT INTO title_watch_providers (title_id, watch_provider_id, country_codes) VALUES ($1, $2, $3)
        ON CONFLICT (title_id, watch_provider_id) DO UPDATE SET country_codes = $3",
        title.id,
        watch_provider.id,
        &country_codes
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn paginate_title_watch_providers(
    cursor_params: &CursorParams,
    title: &Title<'_>,
    country_code: Option<&str>,
) -> CursorPage<TitleWatchProvider> {
    let db_pool = db_pool().await;

    CursorPage::new(
        cursor_params,
        |node: &TitleWatchProvider| node.id,
        async |after| get_title_watch_provider_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let cursor_id = cursor_resource.map(|r| Some(r.id)).unwrap_or_default();

            sqlx::query_as!(
                TitleWatchProvider,
                r#"SELECT * FROM title_watch_providers
                WHERE
                    ($1::uuid IS NULL OR id > $1) AND title_id = $2 AND ($3::text IS NULL OR $3 = ANY(country_codes))
                ORDER BY id ASC LIMIT $4"#,
                cursor_id,    // $1
                title.id,     // $2
                country_code, // $3
                limit         // $4
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}

pub async fn paginate_watch_providers<'a>(
    cursor_params: CursorParams,
    country_code: Option<&str>,
) -> CursorPage<WatchProvider<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        &cursor_params,
        |node: &WatchProvider| node.id,
        async |after| get_watch_provider_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let (cursor_id, cursor_name) = cursor_resource
                .map(|r| (Some(r.id), Some(r.name.to_string())))
                .unwrap_or_default();

            sqlx::query_as!(
                WatchProvider,
                r#"SELECT * FROM watch_providers AS w
                WHERE
                    ($1::uuid IS NULL OR (name, id) > ($2, $1))
                    AND (
                        $3::text IS NULL
                        OR (
                            SELECT id FROM title_watch_providers
                            WHERE watch_provider_id = w.id AND $3 = ANY(country_codes) LIMIT 1
                        ) IS NOT NULL
                    )
                ORDER BY name ASC, id ASC LIMIT $4"#,
                cursor_id,    // $1
                cursor_name,  // $2
                country_code, // $3
                limit         // $4
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}
