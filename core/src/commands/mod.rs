use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use cached::async_sync::OnceCell;
use cached::{AsyncRedisCache, IOCachedAsync};
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::config::CACHE_CONFIG;

mod genre_commands;
mod keyword_commands;
mod person_commands;
mod session_commands;
mod title_commands;
mod user_commands;

pub use person_commands::*;
pub use genre_commands::*;
pub use keyword_commands::*;
pub use session_commands::*;
pub use title_commands::*;
pub use user_commands::*;

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
pub(crate) trait AsyncRedisCacheExt<K> {
    async fn cache_remove(&self, prefix: &str, key: &K);
}

impl<K, V> AsyncRedisCacheExt<K> for OnceCell<AsyncRedisCache<K, V>>
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

async fn download_file(source_url: Url, dest_path: PathBuf) -> anyhow::Result<()> {
    let bytes = reqwest::get(source_url).await?.bytes().await?;

    let mut file = File::create(dest_path)?;

    file.write_all(&bytes)?;

    Ok(())
}

fn generate_random_string(length: u8) -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(length as usize)
        .map(char::from)
        .collect()
}
