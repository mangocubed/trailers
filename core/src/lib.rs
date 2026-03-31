use apalis::prelude::TaskSink;
use apalis_redis::RedisStorage;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::OnceCell;

mod constants;
mod pagination;

#[cfg(feature = "graphql")]
pub mod graphql;

pub mod commands;
pub mod config;
pub mod enums;
pub mod identity;
pub mod jobs;
pub mod models;

use crate::config::{DATABASE_CONFIG, MONITOR_CONFIG, SENTRY_CONFIG};
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

pub fn start_tracing_subscriber() -> Option<sentry::ClientInitGuard> {
    use sentry::integrations::tracing::EventFilter;
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::prelude::*;

    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(if cfg!(debug_assertions) {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    });

    let sentry_guard = if let Some(sentry_dsn) = SENTRY_CONFIG.dsn.as_deref() {
        let guard = sentry::init((
            sentry_dsn,
            sentry::ClientOptions {
                debug: cfg!(debug_assertions),
                enable_logs: true,
                release: Some(env!("CARGO_PKG_VERSION").into()),
                traces_sample_rate: SENTRY_CONFIG.traces_sample_rate,
                send_default_pii: SENTRY_CONFIG.send_default_pii,
                ..Default::default()
            },
        ));

        let sentry_layer = sentry::integrations::tracing::layer()
            .event_filter(|metadata| match *metadata.level() {
                tracing::Level::ERROR => EventFilter::Event | EventFilter::Log,
                tracing::Level::WARN => EventFilter::Breadcrumb | EventFilter::Log,
                _ => EventFilter::Ignore,
            })
            .span_filter(|metadata| matches!(*metadata.level(), tracing::Level::ERROR | tracing::Level::WARN));

        tracing_subscriber::registry().with(fmt_layer).with(sentry_layer).init();

        Some(guard)
    } else {
        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(tracing_subscriber::fmt::layer())
            .init();

        None
    };

    tracing::info!("Tracing subscriber initialized.");

    sentry_guard
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

    pub(crate) async fn push_new_user(&self, user: &User) {
        self.new_user
            .clone()
            .push(NewUserJob { user_id: user.id })
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
