use apalis::prelude::TaskSink;
use apalis_redis::RedisStorage;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::OnceCell;

use toolbox::identity_client::IdentityClient;

mod constants;

#[cfg(feature = "graphql")]
pub mod graphql;

pub mod commands;
pub mod config;
pub mod enums;
pub mod jobs;
pub mod models;

use crate::config::{DATABASE_CONFIG, MONITOR_CONFIG};
use crate::jobs::{GenerateVideoHlsJob, NewUserJob, PopulateJob, TitleRecommendationsJob};
use crate::models::{Title, User, Video};

static DB_POOL_CELL: OnceCell<PgPool> = OnceCell::const_new();
static JOBS_STORAGE_CELL: OnceCell<JobsStorage> = OnceCell::const_new();

async fn db_pool<'a>() -> &'a PgPool {
    DB_POOL_CELL
        .get_or_init(|| async {
            PgPoolOptions::new()
                .max_connections(DATABASE_CONFIG.max_connections)
                .connect(&DATABASE_CONFIG.url)
                .await
                .expect("Could not create DB pool.")
        })
        .await
}

pub async fn jobs_storage<'a>() -> &'a JobsStorage {
    JOBS_STORAGE_CELL
        .get_or_init(|| async { JobsStorage::new().await })
        .await
}

pub struct JobsStorage {
    pub generate_video_hls: RedisStorage<GenerateVideoHlsJob>,
    pub new_user: RedisStorage<NewUserJob>,
    pub populate: RedisStorage<PopulateJob>,
    pub title_recommendations: RedisStorage<TitleRecommendationsJob>,
}

impl JobsStorage {
    async fn new() -> Self {
        Self {
            generate_video_hls: Self::storage().await,
            new_user: Self::storage().await,
            populate: Self::storage().await,
            title_recommendations: Self::storage().await,
        }
    }

    async fn storage<T: Serialize + for<'de> Deserialize<'de>>() -> RedisStorage<T> {
        let conn = apalis_redis::connect(MONITOR_CONFIG.redis_url.clone())
            .await
            .expect("Could not connect to Redis Jobs DB");

        RedisStorage::new(conn)
    }

    pub(crate) async fn push_generate_video_hls(&self, video: &Video<'_>) {
        self.generate_video_hls
            .clone()
            .push(GenerateVideoHlsJob { video_id: video.id })
            .await
            .expect("Could not store job");
    }

    pub(crate) async fn push_new_user(&self, identity_client: &IdentityClient, user: &User) {
        self.new_user
            .clone()
            .push(NewUserJob {
                identity_client: identity_client.clone(),
                user_id: user.id,
            })
            .await
            .expect("Could not store job");
    }

    pub async fn push_populate(
        &self,
        query: Option<String>,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) {
        self.populate
            .clone()
            .push(PopulateJob {
                query,
                start_date,
                end_date,
            })
            .await
            .expect("Could not store job");
    }

    pub async fn push_title_recommendations(&self, user: &User, title: &Title<'_>) {
        self.title_recommendations
            .clone()
            .push(TitleRecommendationsJob {
                user_id: user.id,
                title_id: title.id,
            })
            .await
            .expect("Could not store job");
    }
}

#[derive(Serialize)]
pub struct Info {
    pub built_at: DateTime<Utc>,
    pub version: String,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            built_at: env!("BUILD_DATETIME").parse().unwrap(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
