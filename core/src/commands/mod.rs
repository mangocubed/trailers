use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use md5::{Digest, Md5};
use url::Url;

use toolbox::cache::redis_cache_store;

use toolbox::identity_client::{IdentityClient, IdentityUser};

use crate::constants::CACHE_PREFIX_GET_IDENTITY_USER;

mod genre_commands;
mod keyword_commands;
mod person_commands;
mod title_cast_commands;
mod title_commands;
mod title_crew_commands;
mod title_recommendation_commands;
mod title_stat_commands;
mod user_commands;
mod user_title_tie_commands;
mod video_commands;
mod watch_provider_commands;

pub use genre_commands::*;
pub use keyword_commands::*;
pub use person_commands::*;
pub use title_cast_commands::*;
pub use title_commands::*;
pub use title_crew_commands::*;
pub use title_recommendation_commands::*;
pub use title_stat_commands::*;
pub use user_commands::*;
pub use user_title_tie_commands::*;
pub use video_commands::*;
pub use watch_provider_commands::*;

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
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_IDENTITY_USER).await }"##
)]
pub async fn get_identity_user(client: &IdentityClient, username_or_id: &str) -> anyhow::Result<IdentityUser<'static>> {
    Ok(client.user(username_or_id).await?)
}
