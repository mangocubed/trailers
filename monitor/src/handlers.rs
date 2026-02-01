use std::borrow::Cow;

use apalis::prelude::BoxDynError;
use serde::Deserialize;

use trailers_core::commands;
use trailers_core::jobs::{NewSessionJob, NewUserJob};

use crate::config::IP_GEO_CONFIG;
use crate::mailer::{admin_emails, send_new_session_email, send_welcome_email};

#[derive(Deserialize)]
struct Location<'a> {
    country_code2: Cow<'a, str>,
    state_prov: Cow<'a, str>,
    city: Cow<'a, str>,
}

#[derive(Deserialize)]
struct IpGeo<'a> {
    location: Location<'a>,
}

pub async fn new_session(job: NewSessionJob) -> Result<(), BoxDynError> {
    let mut session = commands::get_session_by_id(job.session_id).await?;

    if !job.ip_addr.is_loopback() && !job.ip_addr.is_multicast() && !job.ip_addr.is_unspecified() {
        let result = reqwest::get(format!(
            "https://api.ipgeolocation.io/v2/ipgeo?apiKey={}&ip={}",
            IP_GEO_CONFIG.api_key, job.ip_addr
        ))
        .await;

        if let Ok(response) = result
            && let Ok(ip_geo) = response.json::<IpGeo>().await
        {
            let result = commands::update_session_location(
                &session,
                &ip_geo.location.country_code2,
                &ip_geo.location.state_prov,
                &ip_geo.location.city,
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
