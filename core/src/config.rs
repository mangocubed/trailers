use std::path::PathBuf;
use std::sync::LazyLock;

use envconfig::Envconfig;
use url::Url;

pub(crate) static DATABASE_CONFIG: LazyLock<DatabaseConfig> =
    LazyLock::new(|| DatabaseConfig::init_from_env().unwrap());
pub(crate) static MONITOR_CONFIG: LazyLock<MonitorConfig> = LazyLock::new(|| MonitorConfig::init_from_env().unwrap());
pub static STORAGE_CONFIG: LazyLock<StorageConfig> = LazyLock::new(|| StorageConfig::init_from_env().unwrap());
pub(crate) static YT_DLP_CONFIG: LazyLock<YtDlpConfig> = LazyLock::new(|| YtDlpConfig::init_from_env().unwrap());

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
    #[envconfig(from = "MONITOR_REDIS_URL", default = "redis://127.0.0.1:6379/1")]
    pub redis_url: String,
}

#[derive(Envconfig)]
pub struct StorageConfig {
    #[envconfig(from = "STORAGE_PATH", default = "./storage/")]
    pub path: PathBuf,
    #[envconfig(from = "STORAGE_URL", default = "http://127.0.0.1:8005/storage/")]
    pub url: Url,
}

#[derive(Envconfig)]
pub struct YtDlpConfig {
    #[envconfig(from = "YT_DLP_PROXY")]
    pub proxy: Option<String>,
}
