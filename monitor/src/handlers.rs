use apalis::prelude::BoxDynError;
use apalis_cron::Tick;

use tracing::info;
use trailers_core::commands;
use trailers_core::jobs::{NewSessionJob, NewUserJob, PopulateJob, VideoRecommendationsJob};

use crate::ip_geo::IpGeo;
use crate::mailer::{admin_emails, send_new_session_email, send_welcome_email};
use crate::populate::{populate_movies, populate_persons, populate_series};

pub async fn daily(_tick: Tick) -> Result<(), BoxDynError> {
    populate(PopulateJob::default()).await?;

    Ok(())
}

pub async fn new_session(job: NewSessionJob) -> Result<(), BoxDynError> {
    let mut session = commands::get_session_by_id(job.session_id).await?;
    let ip_geo = IpGeo::new();

    if !job.ip_addr.is_loopback() && !job.ip_addr.is_multicast() && !job.ip_addr.is_unspecified() {
        let result = ip_geo.info(job.ip_addr).await;

        if let Ok(ip_geo_info) = result {
            let result = commands::update_session_location(
                &session,
                &ip_geo_info.location.country_code2,
                &ip_geo_info.location.state_prov,
                &ip_geo_info.location.city,
            )
            .await;

            if let Ok(updated_session) = result {
                session = updated_session
            }
        }
    };

    send_new_session_email(&session).await
}

pub async fn new_user(job: NewUserJob) -> Result<(), BoxDynError> {
    let user = commands::get_user_by_id(job.user_id).await?;

    let _ = admin_emails::send_new_user_email(&user).await;

    send_welcome_email(&user).await
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

pub async fn video_recommendations_handler(job: VideoRecommendationsJob) -> Result<(), BoxDynError> {
    let user = commands::get_user_by_id(job.user_id).await?;

    commands::update_video_recommendations(&user).await?;

    Ok(())
}
