use std::time::Duration;

use apalis::layers::WorkerBuilderExt;
use apalis::layers::sentry::SentryLayer;
use apalis::prelude::{Monitor, WorkerBuilder};
use apalis_cron::CronStream;
use apalis_cron::builder::schedule;
use sentry::integrations::tower::NewSentryLayer;
use tokio::signal::unix::SignalKind;
use tracing::info;

use toolbox::tracing::start_tracing_subscriber;

use trailers_core::jobs_storage;

mod config;
mod handlers;
mod mailer;
mod populate;
mod tmdb;

#[tokio::main]
async fn main() {
    let _guard = start_tracing_subscriber();

    info!("Monitor starting");

    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt()).expect("Could not create sigint listener");
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate()).expect("Could not create sigterm listener");

    let jobs_storage = jobs_storage().await;

    let daily_worker = |index| {
        let daily_schedule = schedule().each().day().at("0:00").build();

        WorkerBuilder::new(format!("daily-{index}"))
            .backend(CronStream::new(daily_schedule))
            .layer(NewSentryLayer::new_from_top())
            .layer(SentryLayer::new())
            .enable_tracing()
            .concurrency(1)
            .build(handlers::daily)
    };

    let generate_video_hls_worker = |index| {
        WorkerBuilder::new(format!("generate-video-hls-{index}"))
            .backend(jobs_storage.generate_video_hls.clone())
            .layer(NewSentryLayer::new_from_top())
            .layer(SentryLayer::new())
            .enable_tracing()
            .concurrency(1)
            .build(handlers::generate_video_hls)
    };

    let new_user_worker = |index| {
        WorkerBuilder::new(format!("new-user-{index}"))
            .backend(jobs_storage.new_user.clone())
            .layer(NewSentryLayer::new_from_top())
            .layer(SentryLayer::new())
            .enable_tracing()
            .concurrency(1)
            .build(handlers::new_user)
    };

    let populate_worker = |index| {
        WorkerBuilder::new(format!("populate-{index}"))
            .backend(jobs_storage.populate.clone())
            .layer(NewSentryLayer::new_from_top())
            .layer(SentryLayer::new())
            .enable_tracing()
            .concurrency(1)
            .build(handlers::populate)
    };

    let title_recommendations_worker = |index| {
        WorkerBuilder::new(format!("title-recommendations-{index}"))
            .backend(jobs_storage.title_recommendations.clone())
            .layer(NewSentryLayer::new_from_top())
            .layer(SentryLayer::new())
            .enable_tracing()
            .concurrency(1)
            .build(handlers::title_recommendations_handler)
    };

    Monitor::new()
        .register(daily_worker)
        .register(generate_video_hls_worker)
        .register(new_user_worker)
        .register(populate_worker)
        .register(title_recommendations_worker)
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
