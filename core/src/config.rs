use std::net::SocketAddr;
use std::sync::LazyLock;
use std::time::Duration;

use envconfig::Envconfig;

pub static API_CONFIG: LazyLock<ApiConfig> = LazyLock::new(|| ApiConfig::init_from_env().unwrap());
pub(crate) static CACHE_CONFIG: LazyLock<CacheConfig> =
    LazyLock::new(|| CacheConfig::init_from_env().unwrap());
pub(crate) static DATABASE_CONFIG: LazyLock<DatabaseConfig> =
    LazyLock::new(|| DatabaseConfig::init_from_env().unwrap());
pub(crate) static MONITOR_CONFIG: LazyLock<MonitorConfig> =
    LazyLock::new(|| MonitorConfig::init_from_env().unwrap());

#[derive(Envconfig)]
pub struct ApiConfig {
    #[envconfig(from = "API_ADDRESS", default = "127.0.0.1:8000")]
    pub address: SocketAddr,
    #[envconfig(from = "API_OLD_TOKENS", default = "")]
    old_tokens: String,
    #[envconfig(from = "API_TOKENS", default = "trailers_app")]
    tokens: String,
}

impl ApiConfig {
    pub fn old_tokens(&self) -> Vec<&str> {
        self.old_tokens
            .split(',')
            .map(|token| token.trim())
            .collect()
    }

    pub fn tokens(&self) -> Vec<&str> {
        self.tokens.split(',').map(|token| token.trim()).collect()
    }
}

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
