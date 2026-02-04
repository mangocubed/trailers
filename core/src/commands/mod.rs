use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use cached::async_sync::OnceCell;
use cached::{AsyncRedisCache, IOCachedAsync};
use md5::{Digest, Md5};
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
mod title_cast_commands;
mod title_commands;
mod title_crew_commands;
mod user_commands;
mod user_title_tie_commands;
mod video_commands;
mod video_recommendation_commands;
mod video_view_commands;
mod watch_provider_commands;

pub use genre_commands::*;
pub use keyword_commands::*;
pub use person_commands::*;
pub use session_commands::*;
pub use title_cast_commands::*;
pub use title_commands::*;
pub use title_crew_commands::*;
pub use user_commands::*;
pub use user_title_tie_commands::*;
pub use video_commands::*;
pub use video_recommendation_commands::*;
pub use video_view_commands::*;
pub use watch_provider_commands::*;

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
    let content = reqwest::get(source_url).await?.bytes().await?;

    let md5_checksum = Md5::digest(&content);

    if dest_path.exists()
        && let Ok(existing_content) = std::fs::read(&dest_path)
    {
        let existing_md5_checksum = Md5::digest(&existing_content);

        if existing_md5_checksum == md5_checksum {
            return Ok(());
        }
    }

    if let Some(parent_dir) = dest_path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }

    let mut file = File::create(dest_path)?;

    file.write_all(&content)?;

    Ok(())
}

fn generate_random_string(length: u8) -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(length as usize)
        .map(char::from)
        .collect()
}
