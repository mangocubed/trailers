use std::borrow::Cow;
use std::fmt::Display;
use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use url::Url;
use uuid::Uuid;

use crate::commands;
use crate::config::STORAGE_CONFIG;
use crate::enums::{TitleCrewJob, TitleMediaType, VideoOrientation, VideoSource, VideoType};
use crate::identity::IdentityUser;

pub struct Genre<'a> {
    pub id: Uuid,
    pub tmdb_id: i32,
    pub name: Cow<'a, str>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub type Keyword<'a> = Genre<'a>;

pub struct Person<'a> {
    pub id: Uuid,
    pub tmdb_id: i32,
    pub tmdb_profile_path: Option<String>,
    pub imdb_id: Option<String>,
    pub name: Cow<'a, str>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Person<'_> {
    pub fn profile_image_path(&self) -> Option<PathBuf> {
        if self.tmdb_profile_path.is_some() {
            Some(
                STORAGE_CONFIG
                    .path
                    .join(format!("person_profiles/original/{}.jpg", self.id)),
            )
        } else {
            None
        }
    }

    pub fn profile_image_url(&self) -> Option<Url> {
        if self.tmdb_profile_path.is_some() {
            STORAGE_CONFIG
                .url
                .join(&format!("person_profiles/original/{}.jpg", self.id))
                .ok()
        } else {
            None
        }
    }
}

pub struct Title<'a> {
    pub id: Uuid,
    pub media_type: TitleMediaType,
    pub tmdb_id: i32,
    pub tmdb_backdrop_path: Option<String>,
    pub tmdb_poster_path: Option<String>,
    pub imdb_id: Option<String>,
    pub name: Cow<'a, str>,
    pub overview: Cow<'a, str>,
    pub language: Cow<'a, str>,
    pub runtime: Option<PgInterval>,
    pub is_adult: bool,
    pub released_on: Option<NaiveDate>,
    pub search_rank: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Title<'_> {
    pub fn backdrop_image_path(&self) -> Option<PathBuf> {
        if self.tmdb_backdrop_path.is_some() {
            Some(
                STORAGE_CONFIG
                    .path
                    .join(format!("title_backdrops/original/{}.jpg", self.id)),
            )
        } else {
            None
        }
    }

    pub fn backdrop_image_url(&self) -> Option<Url> {
        if self.tmdb_backdrop_path.is_some() {
            STORAGE_CONFIG
                .url
                .join(&format!("title_backdrops/original/{}.jpg", self.id))
                .ok()
        } else {
            None
        }
    }

    pub fn poster_image_path(&self) -> Option<PathBuf> {
        if self.tmdb_poster_path.is_some() {
            Some(
                STORAGE_CONFIG
                    .path
                    .join(format!("title_posters/original/{}.jpg", self.id)),
            )
        } else {
            None
        }
    }

    pub fn poster_image_url(&self) -> Option<Url> {
        if self.tmdb_poster_path.is_some() {
            STORAGE_CONFIG
                .url
                .join(&format!("title_posters/original/{}.jpg", self.id))
                .ok()
        } else {
            None
        }
    }
}

pub struct TitleCast<'a> {
    pub id: Uuid,
    pub title_id: Uuid,
    pub person_id: Uuid,
    pub tmdb_credit_id: Cow<'a, str>,
    pub character_name: Cow<'a, str>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct TitleCrew<'a> {
    pub id: Uuid,
    pub title_id: Uuid,
    pub person_id: Uuid,
    pub tmdb_credit_id: Cow<'a, str>,
    pub job: TitleCrewJob,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct TitleWatchProvider {
    pub id: Uuid,
    pub title_id: Uuid,
    pub watch_provider_id: Uuid,
    pub country_codes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    pub identity_user_id: Uuid,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl User {
    pub async fn identity_user(&self) -> anyhow::Result<IdentityUser<'_>> {
        commands::get_identity_user(&self.identity_user_id.to_string()).await
    }
}

pub struct UserTitleTie {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title_id: Uuid,
    pub bookmarked_at: Option<DateTime<Utc>>,
    pub bookmarked_video_id: Option<Uuid>,
    pub liked_at: Option<DateTime<Utc>>,
    pub liked_video_id: Option<Uuid>,
    pub watched_at: Option<DateTime<Utc>>,
    pub watched_video_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl UserTitleTie {
    pub async fn title(&self) -> sqlx::Result<Title<'_>> {
        commands::get_title_by_id(self.title_id, None).await
    }

    pub async fn user(&self) -> sqlx::Result<User> {
        commands::get_user_by_id(self.user_id).await
    }
}

pub struct Video<'a> {
    pub id: Uuid,
    pub title_id: Uuid,
    pub tmdb_id: Cow<'a, str>,
    pub source: VideoSource,
    pub source_key: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub video_type: VideoType,
    pub duration_secs: i32,
    pub orientation: VideoOrientation,
    pub language: Cow<'a, str>,
    pub relevance: i64,
    pub published_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Video<'_> {
    pub fn path(&self) -> PathBuf {
        STORAGE_CONFIG.path.join(format!("videos/original/{}.mp4", self.id))
    }

    pub async fn title(&self) -> sqlx::Result<Title<'_>> {
        commands::get_title_by_id(self.title_id, None).await
    }

    pub fn url(&self) -> Url {
        STORAGE_CONFIG
            .url
            .join(&format!("videos/original/{}.mp4", self.id))
            .unwrap()
    }
}

pub struct WatchProvider<'a> {
    pub id: Uuid,
    pub tmdb_id: i32,
    pub tmdb_logo_path: Option<String>,
    pub name: Cow<'a, str>,
    pub home_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl WatchProvider<'_> {
    pub fn home_url(&self) -> Option<Url> {
        self.home_url.as_ref().and_then(|u| u.parse().ok())
    }

    pub fn logo_image_path(&self) -> Option<PathBuf> {
        if self.tmdb_logo_path.is_some() {
            Some(
                STORAGE_CONFIG
                    .path
                    .join(format!("watch_provider_logos/original/{}.jpg", self.id)),
            )
        } else {
            None
        }
    }

    pub fn logo_image_url(&self) -> Option<Url> {
        if self.tmdb_logo_path.is_some() {
            STORAGE_CONFIG
                .url
                .join(&format!("watch_provider_logos/original/{}.jpg", self.id))
                .ok()
        } else {
            None
        }
    }
}
