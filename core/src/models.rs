use std::borrow::Cow;
use std::fmt::Display;
use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use url::Url;
use uuid::Uuid;

use crate::config::STORAGE_CONFIG;
use crate::enums::{TitleCrewJob, TitleMediaType, VideoOrientation, VideoSource, VideoType};
use crate::identity_client::{IdentityClient, IdentityUser};
use crate::{commands, jobs_storage};

pub struct Genre<'a> {
    pub id: Uuid,
    pub tmdb_id: i32,
    pub name: Cow<'a, str>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Interval(pub Option<TimeDelta>);

impl From<Option<PgInterval>> for Interval {
    fn from(value: Option<PgInterval>) -> Self {
        Self(value.map(|v| TimeDelta::microseconds(v.microseconds)))
    }
}

pub type Keyword<'a> = Genre<'a>;

#[derive(Clone, Deserialize, Serialize)]
pub struct Person<'a> {
    pub id: Uuid,
    pub tmdb_id: i32,
    pub tmdb_profile_path: Option<String>,
    pub imdb_id: Option<String>,
    pub name: Cow<'a, str>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for Person<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
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

#[derive(Clone, Deserialize, Serialize)]
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
    pub runtime: Interval,
    pub released_on: Option<NaiveDate>,
    pub relevance: i64,
    pub popularity: i64,
    pub search_rank: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for Title<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
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

    pub async fn stat(&self) -> sqlx::Result<TitleStat> {
        commands::get_or_insert_title_stat(self).await
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

pub struct TitleStat {
    pub id: Uuid,
    pub title_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct TitleWatchProvider {
    pub id: Uuid,
    pub title_id: Uuid,
    pub watch_provider_id: Uuid,
    pub country_codes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for TitleWatchProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
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
    pub async fn identity_user(&self, client: &IdentityClient) -> anyhow::Result<IdentityUser<'_>> {
        commands::get_identity_user(client, &self.identity_user_id.to_string()).await
    }
}

pub struct UserTitleTie {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title_id: Uuid,
    pub bookmarked_at: Option<DateTime<Utc>>,
    pub liked_at: Option<DateTime<Utc>>,
    pub watched_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl UserTitleTie {
    pub async fn title(&self) -> sqlx::Result<Title<'_>> {
        commands::get_title_by_id(self.title_id, None, None).await
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
    pub published_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Video<'_> {
    pub fn hls_path(&self) -> PathBuf {
        STORAGE_CONFIG
            .path
            .join(format!("videos/hls/{}/playlist.m3u8", self.id))
    }

    pub async fn hls_url(&self) -> Option<Url> {
        if std::fs::exists(self.hls_path()).unwrap_or_default() {
            Some(
                STORAGE_CONFIG
                    .url
                    .join(&format!("videos/hls/{}/playlist.m3u8", self.id))
                    .unwrap(),
            )
        } else {
            jobs_storage().await.push_generate_video_hls(self).await;

            None
        }
    }

    pub fn path(&self) -> PathBuf {
        STORAGE_CONFIG.path.join(format!("videos/original/{}.mp4", self.id))
    }

    pub async fn title(&self) -> sqlx::Result<Title<'_>> {
        commands::get_title_by_id(self.title_id, None, None).await
    }

    pub fn url(&self) -> Url {
        STORAGE_CONFIG
            .url
            .join(&format!("videos/original/{}.mp4", self.id))
            .unwrap()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct WatchProvider<'a> {
    pub id: Uuid,
    pub tmdb_id: i32,
    pub tmdb_logo_path: Option<String>,
    pub name: Cow<'a, str>,
    pub home_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for WatchProvider<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
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
