use std::sync::LazyLock;

use envconfig::Envconfig;

pub static API_CONFIG: LazyLock<ApiConfig> = LazyLock::new(|| ApiConfig::init_from_env().unwrap());

#[derive(Envconfig)]
pub struct ApiConfig {
    #[envconfig(from = "API_ADDRESS", default = "127.0.0.1:8000")]
    pub address: String,
}
