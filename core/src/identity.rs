use std::borrow::Cow;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use reqwest::Result;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::config::IDENTITY_CONFIG;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IdentityUser<'a> {
    pub id: Uuid,
    pub username: Cow<'a, str>,
    pub email: Cow<'a, str>,
    pub display_name: Cow<'a, str>,
    pub initials: Cow<'a, str>,
    pub language_code: Cow<'a, str>,
    pub country_code: Cow<'a, str>,
    pub avatar_image_url: Url,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for IdentityUser<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

pub struct Identity {
    api_url: Url,
    token: Option<String>,
}

impl Default for Identity {
    fn default() -> Self {
        Self {
            api_url: IDENTITY_CONFIG.api_url.clone(),
            token: None,
        }
    }
}

impl Identity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_token(mut self, token: String) -> Self {
        self.token = Some(token);

        self
    }

    fn request_url(&self, path: &str) -> Url {
        self.api_url.join(path).unwrap()
    }

    pub async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.get_with_query(path, None).await
    }

    pub async fn get_with_query<T>(&self, path: &str, query: Option<&str>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut request_url = self.request_url(path);

        request_url.set_query(query);

        let mut client = reqwest::Client::new().get(request_url.clone());

        if let Some(token) = &self.token {
            client = client.bearer_auth(token);
        }

        let result = client.send().await?.error_for_status()?.json().await;

        match result {
            Ok(data) => Ok(data),
            Err(err) => {
                tracing::error!("Could not execute request: {:?}", err);

                Err(err)
            }
        }
    }

    pub async fn authorized(&self) -> Result<String> {
        self.get_with_query("/authorized", None).await
    }

    pub async fn current_user(&self) -> Result<IdentityUser<'_>> {
        self.get_with_query("/current-user", None).await
    }

    pub async fn user<'a>(&self, username_or_id: &str) -> Result<IdentityUser<'a>> {
        self.get_with_query(&format!("/users/{username_or_id}"), None).await
    }
}
