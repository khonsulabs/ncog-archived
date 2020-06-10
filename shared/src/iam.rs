use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum IAMRequest {
    UsersList,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum IAMResponse {
    UsersList(Vec<User>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: i64,
    pub screenname: Option<String>,
    pub created_at: DateTime<Utc>,
    pub roles: Vec<RoleSummary>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoleSummary {
    pub id: i64,
    pub name: String,
}
