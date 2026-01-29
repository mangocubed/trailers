use std::fmt::Display;

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use cached::async_sync::OnceCell;
use cached::proc_macro::io_cached;
use cached::{AsyncRedisCache, IOCachedAsync};
use chrono::NaiveDate;
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::config::CACHE_CONFIG;
use crate::constants::{CACHE_PREFIX_GET_USER_ID_BY_EMAIL, CACHE_PREFIX_GET_USER_ID_BY_USERNAME};
use crate::models::User;
use crate::{db_pool, jobs_storage};

async fn async_redis_cache<K, V>(prefix: &str) -> AsyncRedisCache<K, V>
where
    K: Display + Send + Sync,
    V: DeserializeOwned + Display + Send + Serialize + Sync,
{
    AsyncRedisCache::new(format!("{prefix}:"), CACHE_CONFIG.ttl())
        .set_connection_string(&CACHE_CONFIG.redis_url)
        .set_refresh(true)
        .build()
        .await
        .expect("Could not get redis cache")
}

#[allow(dead_code)]
pub(crate) trait AsyncRedisCacheTrait<K> {
    async fn cache_remove(&self, prefix: &str, key: &K);
}

impl<K, V> AsyncRedisCacheTrait<K> for OnceCell<AsyncRedisCache<K, V>>
where
    K: Display + Send + Sync,
    V: DeserializeOwned + Display + Send + Serialize + Sync,
{
    async fn cache_remove(&self, prefix: &str, key: &K) {
        let _ = self
            .get_or_init(|| async { async_redis_cache(prefix).await })
            .await
            .cache_remove(key)
            .await;
    }
}

pub async fn email_exists(email: &str) -> bool {
    get_user_id_by_email(email).await.is_ok()
}

fn encrypt_password(value: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(value.as_bytes(), &salt).unwrap().to_string()
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ email.to_lowercase() }"#,
    ty = "cached::AsyncRedisCache<String, Uuid>",
    create = r##"{ async_redis_cache(CACHE_PREFIX_GET_USER_ID_BY_EMAIL).await }"##
)]
pub async fn get_user_id_by_email(email: &str) -> sqlx::Result<Uuid> {
    if email.is_empty() {
        return Err(sqlx::Error::InvalidArgument("email".to_owned()));
    }

    let db_pool = db_pool().await;

    sqlx::query!(
        r#"SELECT id AS "id!" FROM users WHERE LOWER(email) = $1 LIMIT 1"#,
        email.to_lowercase() // $1
    )
    .fetch_one(db_pool)
    .await
    .map(|record| record.id)
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ username.to_lowercase() }"#,
    ty = "cached::AsyncRedisCache<String, Uuid>",
    create = r##"{ async_redis_cache(CACHE_PREFIX_GET_USER_ID_BY_USERNAME).await }"##
)]
pub async fn get_user_id_by_username(username: &str) -> sqlx::Result<Uuid> {
    if username.is_empty() {
        return Err(sqlx::Error::InvalidArgument("username".to_owned()));
    }

    let db_pool = db_pool().await;

    sqlx::query!(
        r#"SELECT id AS "id!" FROM users WHERE LOWER(username) = $1 LIMIT 1"#,
        username.to_lowercase()
    )
    .fetch_one(db_pool)
    .await
    .map(|record| record.id)
}

/// Attempts to insert an user into the database without making validations.
pub async fn insert_user<'a>(
    username: &str,
    email: &str,
    password: &str,
    full_name: &str,
    birthdate: NaiveDate,
    country_alpha2: &str,
) -> sqlx::Result<User<'a>> {
    let db_pool = db_pool().await;
    let display_name = full_name.split(' ').next().unwrap();
    let encrypted_password = encrypt_password(password);

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (
            username,
            email,
            encrypted_password,
            display_name,
            full_name,
            birthdate,
            country_code
        ) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        username,             // $1
        email.to_lowercase(), // $2
        encrypted_password,   // $3
        display_name,         // $4
        full_name,            // $5
        birthdate,            // $6
        country_alpha2,       // $7
    )
    .fetch_one(db_pool)
    .await?;

    jobs_storage().await.push_new_user(&user).await;

    Ok(user)
}

pub async fn username_exists(username: &str) -> bool {
    get_user_id_by_username(username).await.is_ok()
}
