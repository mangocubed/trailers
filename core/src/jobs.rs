use std::net::IpAddr;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct NewSessionJob {
    pub session_id: Uuid,
    pub ip_addr: IpAddr,
}

#[derive(Deserialize, Serialize)]
pub struct NewUserJob {
    pub user_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct PopulateTitlesJob {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}
