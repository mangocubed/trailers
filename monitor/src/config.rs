use std::sync::LazyLock;

use envconfig::Envconfig;

pub static TMDB_CONFIG: LazyLock<TmdbConfig> = LazyLock::new(|| TmdbConfig::init_from_env().unwrap());

#[derive(Envconfig)]
pub struct TmdbConfig {
    #[envconfig(from = "TMDB_API_KEY", default = "")]
    pub api_key: String,
}
