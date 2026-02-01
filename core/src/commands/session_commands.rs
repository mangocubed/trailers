use std::net::IpAddr;

use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use tokio::join;
use uuid::Uuid;

use crate::commands::GET_USER_BY_SESSION_TOKEN;
use crate::config::USERS_CONFIG;
use crate::constants::*;
use crate::models::{Session, User};
use crate::{db_pool, jobs_storage};

use super::{AsyncRedisCacheExt, async_redis_cache, generate_random_string};

pub async fn finish_session(session: &Session<'_>) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "UPDATE sessions SET finished_at = current_timestamp WHERE finished_at IS NULL AND id = $1",
        session.id
    )
    .execute(db_pool)
    .await?;

    remove_session_cache(session).await;

    Ok(())
}

async fn generate_session_token() -> String {
    let db_pool = db_pool().await;
    let mut token = String::new();
    let mut exists = true;

    while exists {
        token = generate_random_string(USERS_CONFIG.session_token_length);

        exists = sqlx::query!(
            "SELECT id FROM sessions WHERE LOWER(token) = $1 OR LOWER(previous_token) = $1 LIMIT 1",
            token // $1
        )
        .fetch_one(db_pool)
        .await
        .is_ok();
    }

    token
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Session<'_>>",
    create = r##"{ async_redis_cache(CACHE_PREFIX_GET_SESSION_BY_ID).await }"##
)]
pub async fn get_session_by_id(id: Uuid) -> sqlx::Result<Session<'static>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Session,
        "SELECT * FROM sessions WHERE expires_at > current_timestamp AND finished_at IS NULL AND id = $1 LIMIT 1",
        id
    )
    .fetch_one(db_pool)
    .await
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<String, Session<'_>>",
    create = r##"{ async_redis_cache(CACHE_PREFIX_GET_SESSION_BY_TOKEN).await }"##
)]
pub async fn get_session_by_token(token: String) -> sqlx::Result<Session<'static>> {
    if token.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        Session,
        "SELECT * FROM sessions
        WHERE
            expires_at > current_timestamp AND finished_at IS NULL
            AND (token = $1 OR (previous_token = $1 AND refreshed_at > current_timestamp - INTERVAL '1 minute'))
        LIMIT 1",
        token
    )
    .fetch_one(db_pool)
    .await
}

pub(crate) async fn insert_session<'a>(user: &User<'_>, ip_addr: IpAddr) -> sqlx::Result<Session<'a>> {
    let db_pool = db_pool().await;

    let token = generate_session_token().await;

    let result = sqlx::query_as!(
        Session,
        "INSERT INTO sessions (user_id, token) VALUES ($1, $2) RETURNING *",
        user.id, // $1
        token,   // $2
    )
    .fetch_one(db_pool)
    .await;

    match result {
        Ok(session) => {
            jobs_storage().await.push_new_session(&session, ip_addr).await;

            Ok(session)
        }
        Err(err) => Err(err),
    }
}

pub async fn update_session_location<'a>(
    session: &Session<'_>,
    country_code: &str,
    region: &str,
    city: &str,
) -> sqlx::Result<Session<'a>> {
    let db_pool = db_pool().await;

    let session = sqlx::query_as!(
        Session,
        "UPDATE sessions SET country_code = $2, region = $3, city = $4 WHERE finished_at IS NULL AND id = $1 RETURNING *",
        session.id,   // $1
        country_code, // $2
        region,       // $3
        city          // $4
    )
    .fetch_one(db_pool)
    .await?;

    remove_session_cache(&session).await;

    Ok(session)
}

pub async fn remove_session_cache(session: &Session<'_>) {
    let token = session.token.to_string();

    join!(
        GET_SESSION_BY_ID.cache_remove(CACHE_PREFIX_GET_SESSION_BY_ID, &session.id),
        GET_SESSION_BY_TOKEN.cache_remove(CACHE_PREFIX_GET_SESSION_BY_TOKEN, &token),
        GET_USER_BY_SESSION_TOKEN.cache_remove(CACHE_PREFIX_GET_USER_BY_SESSION_TOKEN, &token)
    );
}
