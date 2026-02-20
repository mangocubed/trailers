use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use cached::async_sync::OnceCell;
use cached::proc_macro::io_cached;
use cached::{AsyncRedisCache, IOCachedAsync};
use md5::{Digest, Md5};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::config::CACHE_CONFIG;
use crate::constants::CACHE_PREFIX_GET_IDENTITY_USER;
use crate::identity::{Identity, IdentityUser};

mod genre_commands;
mod keyword_commands;
mod person_commands;
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

#[io_cached(
    map_error = r##"|_| anyhow::anyhow!("Could not get identity user")"##,
    convert = r#"{ username_or_id.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, IdentityUser<'_>>",
    create = r##"{ async_redis_cache(CACHE_PREFIX_GET_IDENTITY_USER).await }"##
)]
pub async fn get_identity_user(username_or_id: &str) -> anyhow::Result<IdentityUser<'static>> {
    let identity_user = Identity::new().user(username_or_id).await?;

    Ok(identity_user)
}
