use apalis::prelude::BoxDynError;
use apalis_cron::Tick;

use tracing::info;
use trailers_core::commands;
use trailers_core::jobs::{NewUserJob, PopulateJob, TitleRecommendationsJob};

use crate::mailer::{admin_emails, send_welcome_email};
use crate::populate::{populate_movies, populate_persons, populate_series};

pub async fn daily(_tick: Tick) -> Result<(), BoxDynError> {
    populate(PopulateJob::default()).await?;

    Ok(())
}

pub async fn new_user(job: NewUserJob) -> Result<(), BoxDynError> {
    let user = commands::get_user_by_id(job.user_id).await?;

    let _ = admin_emails::send_new_user_email(&job.identity_client, &user).await;

    send_welcome_email(&job.identity_client, &user).await?;

    Ok(())
}

pub async fn populate(job: PopulateJob) -> Result<(), BoxDynError> {
    info!("Populating Movies...");
    let _ = populate_movies(&job).await;

    info!("Populating Series...");
    let _ = populate_series(&job).await;

    info!("Populating persons...");
    let _ = populate_persons(&job).await;

    info!("Done!");

    Ok(())
}

pub async fn title_recommendations_handler(job: TitleRecommendationsJob) -> Result<(), BoxDynError> {
    let user = commands::get_user_by_id(job.user_id).await?;
    let title = commands::get_title_by_id(job.title_id, None, None).await?;
    let title_stat = title.stat().await?;

    let _ = commands::update_title_stat(&title_stat).await;
    let _ = commands::update_title_recommendations(&user).await;

    Ok(())
}
