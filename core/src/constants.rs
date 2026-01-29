use std::sync::LazyLock;

use regex::Regex;

pub static REGEX_EMAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A[\w.-]+@[[:alnum:].-]+(\.[[:alpha:]]{2,})+\z").unwrap());

pub static REGEX_USERNAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\A[-_.]?([[:alnum:]]+[-_.]?)+\z").unwrap());

pub const CACHE_PREFIX_GET_USER_ID_BY_EMAIL: &str = "get_user_id_by_email";
pub const CACHE_PREFIX_GET_USER_ID_BY_USERNAME: &str = "get_user_id_by_username";
