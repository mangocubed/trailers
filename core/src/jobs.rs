use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct NewUserJob {
    pub user_id: Uuid,
}

#[derive(Default, Deserialize, Serialize)]
pub struct PopulateJob {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Deserialize, Serialize)]
pub struct TitleRecommendationsJob {
    pub user_id: Uuid,
}
