use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewSessionJob {
    pub session_id: Uuid,
    pub ip_addr: IpAddr,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewUserJob {
    pub user_id: Uuid,
}
