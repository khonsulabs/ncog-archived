use crate::permissions::Claim;
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum IAMRequest {
    UsersList,
    UsersGetProfile(i64),
    RolesList,
    RoleGet(i64),
    RoleSave(RoleSummary),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum IAMResponse {
    UsersList(Vec<User>),
    RolesList(Vec<RoleSummary>),
    UserProfile(User),
    Role(RoleSummary),
    RoleSaved(i64),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: i64,
    pub screenname: Option<String>,
    pub created_at: DateTime<Utc>,
    pub roles: Vec<RoleSummary>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RoleSummary {
    pub id: Option<i64>,
    pub name: String,
}

pub fn roles_list_claim() -> Claim {
    Claim::new("iam", Some("roles"), None, "list")
}

pub fn roles_read_claim(id: Option<i64>) -> Claim {
    Claim::new("iam", Some("roles"), id, "read")
}

pub fn roles_update_claim(id: Option<i64>) -> Claim {
    Claim::new("iam", Some("roles"), id, "update")
}
