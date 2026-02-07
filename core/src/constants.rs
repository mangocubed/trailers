#[cfg(feature = "graphql")]
use std::sync::LazyLock;

#[cfg(feature = "graphql")]
use regex::Regex;

#[cfg(feature = "graphql")]
pub static REGEX_EMAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A[\w.-]+@[[:alnum:].-]+(\.[[:alpha:]]{2,})+\z").unwrap());

#[cfg(feature = "graphql")]
pub static REGEX_USERNAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\A[-_.]?([[:alnum:]]+[-_.]?)+\z").unwrap());

pub const CACHE_PREFIX_GET_SESSION_BY_ID: &str = "get_session_by_id";
pub const CACHE_PREFIX_GET_SESSION_BY_TOKEN: &str = "get_session_by_token";
pub const CACHE_PREFIX_GET_USER_BY_ID: &str = "get_user_by_id";
pub const CACHE_PREFIX_GET_USER_BY_SESSION_TOKEN: &str = "get_user_by_session_token";
pub const CACHE_PREFIX_GET_USER_BY_USERNAME: &str = "get_user_by_username";
pub const CACHE_PREFIX_GET_USER_BY_USERNAME_OR_EMAIL: &str = "get_user_by_username_or_email";
pub const CACHE_PREFIX_GET_USER_ID_BY_EMAIL: &str = "get_user_id_by_email";
pub const CACHE_PREFIX_GET_USER_ID_BY_USERNAME: &str = "get_user_id_by_username";
