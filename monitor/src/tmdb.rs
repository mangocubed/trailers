use std::collections::HashMap;
use std::{borrow::Cow, sync::OnceLock};

use chrono::{DateTime, Days, NaiveDate, Utc};
use reqwest::Result;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::config::TMDB_CONFIG;

#[derive(Deserialize, Debug)]
pub struct TmdbCast<'a> {
    pub id: i32,
    pub credit_id: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub profile_path: Option<Cow<'a, str>>,
    pub character: String,
}

impl TmdbCast<'_> {
    pub fn profile_url(&self) -> Option<Url> {
        self.profile_path.as_deref().map(Tmdb::image_url)
    }
}

#[derive(Deserialize)]
pub struct TmdbCredits<'a> {
    pub cast: Vec<TmdbCast<'a>>,
    #[allow(dead_code)]
    pub crew: Vec<TmdbCrew<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct TmdbCrew<'a> {
    pub id: i32,
    pub credit_id: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub profile_path: Option<Cow<'a, str>>,
    pub job: Cow<'a, str>,
}

impl TmdbCrew<'_> {
    pub fn profile_url(&self) -> Option<Url> {
        self.profile_path.as_deref().map(Tmdb::image_url)
    }
}

#[derive(Deserialize, Clone)]
pub struct TmdbGenre<'a> {
    pub id: i32,
    pub name: Cow<'a, str>,
}

#[derive(Deserialize)]
pub struct TmdbItems {
    pub results: Vec<TmdbItem>,
    pub total_pages: usize,
}

#[derive(Deserialize)]
pub struct TmdbItem {
    pub id: i32,
    pub adult: bool,
}

pub type TmdbKeyword<'a> = TmdbGenre<'a>;

#[derive(Deserialize)]
pub struct TmdbKeywords<'a> {
    pub keywords: Option<Vec<TmdbKeyword<'a>>>,
    pub results: Option<Vec<TmdbKeyword<'a>>>,
}

impl TmdbKeywords<'_> {
    pub fn keywords(&self) -> Vec<TmdbKeyword<'_>> {
        self.keywords
            .clone()
            .or_else(|| self.results.clone())
            .unwrap_or_default()
    }
}

#[derive(Deserialize)]
pub struct TmdbMovie<'a> {
    pub id: i32,
    pub imdb_id: Option<Cow<'a, str>>,
    pub title: Cow<'a, str>,
    pub overview: Cow<'a, str>,
    pub original_language: Cow<'a, str>,
    pub backdrop_path: Option<Cow<'a, str>>,
    pub poster_path: Option<Cow<'a, str>>,
    pub runtime: i64,
    pub adult: Option<bool>,
    pub release_date: Option<Cow<'a, str>>,
    pub genres: Vec<TmdbGenre<'a>>,
}

impl TmdbMovie<'_> {
    pub fn backdrop_url(&self) -> Option<Url> {
        self.backdrop_path.as_deref().map(Tmdb::image_url)
    }

    pub fn poster_url(&self) -> Option<Url> {
        self.poster_path.as_deref().map(Tmdb::image_url)
    }
}

#[derive(Deserialize)]
pub struct TmdbPerson<'a> {
    pub id: i32,
    pub imdb_id: Option<Cow<'a, str>>,
    pub name: Cow<'a, str>,
    pub profile_path: Option<Cow<'a, str>>,
}

impl TmdbPerson<'_> {
    pub fn profile_url(&self) -> Option<Url> {
        self.profile_path.as_deref().map(Tmdb::image_url)
    }
}

#[derive(Deserialize)]
pub struct TmdbTV<'a> {
    pub id: i32,
    pub imdb_id: Option<Cow<'a, str>>,
    pub name: Cow<'a, str>,
    pub overview: Cow<'a, str>,
    pub original_language: Cow<'a, str>,
    pub backdrop_path: Option<Cow<'a, str>>,
    pub poster_path: Option<Cow<'a, str>>,
    pub adult: Option<bool>,
    pub first_air_date: Option<Cow<'a, str>>,
    pub genres: Vec<TmdbGenre<'a>>,
}

impl TmdbTV<'_> {
    pub fn backdrop_url(&self) -> Option<Url> {
        self.backdrop_path.as_deref().map(Tmdb::image_url)
    }

    pub fn poster_url(&self) -> Option<Url> {
        self.poster_path.as_deref().map(Tmdb::image_url)
    }
}

#[derive(Deserialize)]
pub struct TmdbVideo<'a> {
    pub id: Cow<'a, str>,
    pub site: Cow<'a, str>,
    pub key: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub r#type: Cow<'a, str>,
    pub iso_639_1: Cow<'a, str>,
    pub official: bool,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct TmdbVideos<'a> {
    pub results: Vec<TmdbVideo<'a>>,
}

#[derive(Clone, Deserialize)]
pub struct TmdbWatchProvider<'a> {
    pub logo_path: Option<Cow<'a, str>>,
    pub provider_id: i32,
    pub provider_name: Cow<'a, str>,
}

impl TmdbWatchProvider<'_> {
    pub fn logo_url(&self) -> Option<Url> {
        self.logo_path.as_deref().map(Tmdb::image_url)
    }
}

#[derive(Clone, Deserialize)]
pub struct TmdbWatchProviders<'a> {
    pub ads: Option<Vec<TmdbWatchProvider<'a>>>,
    pub flatrate: Option<Vec<TmdbWatchProvider<'a>>>,
}

#[derive(Deserialize)]
pub struct TmdbWatchProvidersByCountry<'a> {
    pub results: HashMap<Cow<'a, str>, TmdbWatchProviders<'a>>,
}

static TMDB: OnceLock<Tmdb> = OnceLock::new();

pub struct Tmdb<'a> {
    api_key: Cow<'a, str>,
}

impl Default for Tmdb<'_> {
    fn default() -> Self {
        Self {
            api_key: Cow::Owned(TMDB_CONFIG.api_key.clone()),
        }
    }
}

impl<'a> Tmdb<'a> {
    pub fn new() -> &'a Self {
        TMDB.get_or_init(Tmdb::default)
    }

    fn request_url(&self, path: &str) -> Url {
        Url::parse(&format!("https://api.themoviedb.org/3/{}", path)).unwrap()
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

        request_url.set_query(Some(&format!("api_key={}&{}", self.api_key, query.unwrap_or_default())));

        let result = reqwest::Client::new()
            .get(request_url.clone())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await;

        match result {
            Ok(data) => Ok(data),
            Err(err) => {
                tracing::error!("Could not execute request: {:?}", err);

                Err(err)
            }
        }
    }

    fn changes_query(&self, page: usize, end_date: Option<NaiveDate>, start_date: Option<NaiveDate>) -> String {
        let end_date = if let Some(end_date) = end_date {
            end_date
        } else {
            Utc::now().date_naive()
        };
        let start_date = if let Some(start_date) = start_date {
            start_date
        } else {
            end_date.checked_sub_days(Days::new(1)).unwrap()
        };

        format!("page={page}&start_date={start_date}&end_date={end_date}")
    }

    pub fn image_url(image_path: &str) -> Url {
        Url::parse(&format!("https://image.tmdb.org/t/p/original/{image_path}")).unwrap()
    }

    pub async fn movie(&self, id: i32) -> Result<TmdbMovie<'_>> {
        self.get(&format!("movie/{id}")).await
    }

    pub async fn movie_changes(
        &self,
        page: usize,
        end_date: Option<NaiveDate>,
        start_date: Option<NaiveDate>,
    ) -> Result<TmdbItems> {
        self.get_with_query("movie/changes", Some(&self.changes_query(page, end_date, start_date)))
            .await
    }

    pub async fn movie_credits(&self, id: i32) -> Result<TmdbCredits<'_>> {
        self.get(&format!("movie/{id}/credits")).await
    }

    pub async fn movie_keywords(&self, id: i32) -> Result<TmdbKeywords<'_>> {
        self.get(&format!("movie/{id}/keywords")).await
    }

    pub async fn movie_videos(&self, id: i32) -> Result<TmdbVideos<'_>> {
        self.get(&format!("movie/{id}/videos")).await
    }

    pub async fn movie_watch_providers(&self, id: i32) -> Result<TmdbWatchProvidersByCountry<'_>> {
        self.get(&format!("movie/{id}/watch/providers")).await
    }

    pub async fn person(&self, id: i32) -> Result<TmdbPerson<'_>> {
        self.get(&format!("person/{id}")).await
    }

    pub async fn person_changes(
        &self,
        page: usize,
        end_date: Option<NaiveDate>,
        start_date: Option<NaiveDate>,
    ) -> Result<TmdbItems> {
        self.get_with_query("person/changes", Some(&self.changes_query(page, end_date, start_date)))
            .await
    }

    pub async fn search_movie(&self, page: usize, query: &str) -> Result<TmdbItems> {
        self.get_with_query("search/movie", Some(&format!("page={page}&query={query}")))
            .await
    }

    pub async fn search_person(&self, page: usize, query: &str) -> Result<TmdbItems> {
        self.get_with_query("search/person", Some(&format!("page={page}&query={query}")))
            .await
    }

    pub async fn search_tv(&self, page: usize, query: &str) -> Result<TmdbItems> {
        self.get_with_query("search/tv", Some(&format!("page={page}&query={query}")))
            .await
    }

    pub async fn tv(&self, id: i32) -> Result<TmdbTV<'_>> {
        self.get(&format!("tv/{id}")).await
    }

    pub async fn tv_changes(
        &self,
        page: usize,
        end_date: Option<NaiveDate>,
        start_date: Option<NaiveDate>,
    ) -> Result<TmdbItems> {
        self.get_with_query("tv/changes", Some(&self.changes_query(page, end_date, start_date)))
            .await
    }

    pub async fn tv_credits(&self, id: i32) -> Result<TmdbCredits<'_>> {
        self.get(&format!("tv/{id}/credits")).await
    }

    pub async fn tv_keywords(&self, id: i32) -> Result<TmdbKeywords<'_>> {
        self.get(&format!("tv/{id}/keywords")).await
    }

    pub async fn tv_videos(&self, id: i32) -> Result<TmdbVideos<'_>> {
        self.get(&format!("tv/{id}/videos")).await
    }

    pub async fn tv_watch_providers(&self, id: i32) -> Result<TmdbWatchProvidersByCountry<'_>> {
        self.get(&format!("tv/{id}/watch/providers")).await
    }
}
