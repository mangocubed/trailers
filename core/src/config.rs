use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;

use envconfig::Envconfig;
use url::Url;

pub(crate) static CACHE_CONFIG: LazyLock<CacheConfig> = LazyLock::new(|| CacheConfig::init_from_env().unwrap());
pub(crate) static DATABASE_CONFIG: LazyLock<DatabaseConfig> =
    LazyLock::new(|| DatabaseConfig::init_from_env().unwrap());
pub(crate) static MONITOR_CONFIG: LazyLock<MonitorConfig> = LazyLock::new(|| MonitorConfig::init_from_env().unwrap());
pub static STORAGE_CONFIG: LazyLock<StorageConfig> = LazyLock::new(|| StorageConfig::init_from_env().unwrap());
pub(crate) static USERS_CONFIG: LazyLock<UsersConfig> = LazyLock::new(|| UsersConfig::init_from_env().unwrap());

#[derive(Envconfig)]
pub struct CacheConfig {
    #[envconfig(from = "CACHE_REDIS_URL", default = "redis://127.0.0.1:6379/1")]
    pub redis_url: String,
    #[envconfig(from = "CACHE_TTL", default = "3600")]
    ttl: u16,
}

impl CacheConfig {
    pub fn ttl(&self) -> Duration {
        Duration::from_secs(self.ttl as u64)
    }
}

#[derive(Envconfig)]
pub(crate) struct DatabaseConfig {
    #[envconfig(from = "DATABASE_MAX_CONNECTIONS", default = "5")]
    pub max_connections: u32,
    #[envconfig(
        from = "DATABASE_URL",
        default = "postgres://mango3:mango3@127.0.0.1:5432/trailers_dev"
    )]
    pub url: String,
}

#[derive(Envconfig)]
pub(crate) struct MonitorConfig {
    #[envconfig(from = "MONITOR_REDIS_URL", default = "redis://127.0.0.1:6379/0")]
    pub redis_url: String,
}

#[derive(Envconfig)]
pub struct StorageConfig {
    #[cfg_attr(not(test), envconfig(from = "STORAGE_PATH", default = "./storage/"))]
    #[cfg_attr(test, envconfig(from = "STORAGE_PATH", default = "./storage/tests/"))]
    pub path: PathBuf,
    #[envconfig(from = "STORAGE_SERVE", default = "true")]
    pub serve: bool,
    #[envconfig(from = "STORAGE_BASE_URL", default = "http://localhost:8080")]
    base_url: Url,
}

impl StorageConfig {
    pub fn url(&self) -> Url {
        self.base_url.join("storage/").unwrap()
    }
}

#[derive(Envconfig)]
pub(crate) struct UsersConfig {
    #[envconfig(from = "USERS_SESSION_TOKEN_LENGTH", default = "64")]
    pub session_token_length: u8,
}
