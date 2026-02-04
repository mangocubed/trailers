use std::time::Duration;

use apalis::layers::WorkerBuilderExt;
use apalis::prelude::{Monitor, WorkerBuilder};
use apalis_cron::CronStream;
use apalis_cron::builder::schedule;
use tokio::signal::unix::SignalKind;
use tracing::{Level, info};

use trailers_core::jobs_storage;

mod config;
mod handlers;
mod ip_geo;
mod mailer;
mod populate;
mod tmdb;

#[tokio::main]
async fn main() {
    let tracing_level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt().with_max_level(tracing_level).init();

    info!("Monitor starting");

    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt()).expect("Could not create sigint listener");
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate()).expect("Could not create sigterm listener");

    let jobs_storage = jobs_storage().await;

    let daily_worker = |index| {
        let daily_schedule = schedule().each().day().at("0:00").build();

        WorkerBuilder::new(format!("daily-{index}"))
            .backend(CronStream::new(daily_schedule))
            .enable_tracing()
            .build(handlers::daily)
    };

    let new_session_worker = |index| {
        WorkerBuilder::new(format!("new-session-{index}"))
            .backend(jobs_storage.new_session.clone())
            .enable_tracing()
            .build(handlers::new_session)
    };

    let new_user_worker = |index| {
        WorkerBuilder::new(format!("new-user-{index}"))
            .backend(jobs_storage.new_user.clone())
            .enable_tracing()
            .build(handlers::new_user)
    };

    let populate_worker = |index| {
        WorkerBuilder::new(format!("populate-{index}"))
            .backend(jobs_storage.populate.clone())
            .enable_tracing()
            .build(handlers::populate)
    };

    let video_recommendations_worker = |index| {
        WorkerBuilder::new(format!("video-recommendations-{index}"))
            .backend(jobs_storage.video_recommendations.clone())
            .enable_tracing()
            .build(video_recommendations_handler)
    };

    Monitor::new()
        .register(daily_worker)
        .register(new_session_worker)
        .register(new_user_worker)
        .register(populate_worker)
        .register(video_recommendations_worker)
        .shutdown_timeout(Duration::from_millis(10000))
        .run_with_signal(async {
            info!("Monitor started");

            tokio::select! {
                _ = sigint.recv() => info!("Received SIGINT."),
                _ = sigterm.recv() => info!("Received SIGTERM."),
            };

            info!("Monitor shutting down");

            Ok(())
        })
        .await
        .expect("Monitor failed");

    info!("Monitor shutdown complete");
}
