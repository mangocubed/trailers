use std::borrow::Cow;
use std::fmt::Display;
use std::path::PathBuf;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use url::Url;
use uuid::Uuid;

use crate::commands;
use crate::config::STORAGE_CONFIG;
use crate::enums::{TitleCrewJob, TitleMediaType};

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
            Some(STORAGE_CONFIG.path.join(format!("person_profiles/{}.jpg", self.id)))
        } else {
            None
        }
    }

    pub fn profile_image_url(&self) -> Option<Url> {
        if self.tmdb_profile_path.is_some() {
            STORAGE_CONFIG
                .url()
                .join(&format!("person_profiles/{}.jpg", self.id))
                .ok()
        } else {
            None
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Session<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: Cow<'a, str>,
    pub previous_token: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub refreshed_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for Session<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Session<'_> {
    pub fn location(&self) -> String {
        let Some(country) = self.country_code.as_ref().and_then(|c| rust_iso3166::from_alpha2(c)) else {
            return "Unknown".to_owned();
        };

        let mut location = country.name.to_owned();

        if let Some(region) = &self.region {
            location += &format!(", {region}");
        }

        if let Some(city) = &self.city {
            location += &format!(", {city}");
        }

        location
    }

    pub async fn user(&self) -> sqlx::Result<User<'_>> {
        commands::get_user_by_id(self.user_id).await
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
            Some(STORAGE_CONFIG.path.join(format!("title_backdrops/{}.jpg", self.id)))
        } else {
            None
        }
    }

    pub fn backdrop_image_url(&self) -> Option<Url> {
        if self.tmdb_backdrop_path.is_some() {
            STORAGE_CONFIG
                .url()
                .join(&format!("title_backdrops/{}.jpg", self.id))
                .ok()
        } else {
            None
        }
    }

    pub fn poster_image_path(&self) -> Option<PathBuf> {
        if self.tmdb_poster_path.is_some() {
            Some(STORAGE_CONFIG.path.join(format!("title_posters/{}.jpg", self.id)))
        } else {
            None
        }
    }

    pub fn poster_image_url(&self) -> Option<Url> {
        if self.tmdb_poster_path.is_some() {
            STORAGE_CONFIG
                .url()
                .join(&format!("title_posters/{}.jpg", self.id))
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

#[derive(Clone, Deserialize, Serialize)]
pub struct User<'a> {
    pub id: Uuid,
    pub username: Cow<'a, str>,
    pub email: Cow<'a, str>,
    pub encrypted_password: Cow<'a, str>,
    pub full_name: Cow<'a, str>,
    pub display_name: Cow<'a, str>,
    pub birthdate: NaiveDate,
    pub language_code: Cow<'a, str>,
    pub country_code: Cow<'a, str>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for User<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl User<'_> {
    pub fn initials(&self) -> String {
        self.display_name
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .collect::<String>()
            .to_uppercase()
    }

    pub fn verify_password(&self, password: &str) -> bool {
        let argon2 = Argon2::default();

        let Ok(password_hash) = PasswordHash::new(&self.encrypted_password) else {
            return false;
        };

        argon2.verify_password(password.as_bytes(), &password_hash).is_ok()
    }
}
