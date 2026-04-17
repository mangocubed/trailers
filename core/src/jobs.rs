use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use toolbox::identity_client::IdentityClient;

#[derive(Deserialize, Serialize)]
pub struct GenerateVideoHlsJob {
    pub video_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct NewUserJob {
    pub identity_client: IdentityClient,
    pub user_id: Uuid,
}

#[derive(Default, Deserialize, Serialize)]
pub struct PopulateJob {
    pub query: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Deserialize, Serialize)]
pub struct TitleRecommendationsJob {
    pub user_id: Uuid,
    pub title_id: Uuid,
}
