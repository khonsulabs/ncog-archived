pub mod migrations;
mod connection;
use shared::{UserProfile, Installation};
use uuid::Uuid;

pub use connection::pg;
pub use sqlx;

use sqlx::executor::RefExecutor;
use sqlx::postgres::Postgres;

pub async fn get_profile<'e, E>(
    executor: E, installation_id: Uuid) -> Result<UserProfile, sqlx::Error> 
    where E: 'e + Send + RefExecutor<'e, Database = Postgres>,{
    sqlx::query_as!(
        UserProfile,
        "SELECT accounts.id, screenname FROM accounts INNER JOIN installations ON installations.account_id = accounts.id WHERE installations.id = $1",
        installation_id,
    )
    .fetch_one(executor)
    .await
}

pub async fn lookup_installation<'e, E>(
    executor: E, installation_id: Uuid) -> Result<Installation, sqlx::Error> 
    where E: 'e + Send + RefExecutor<'e, Database = Postgres>,{
    sqlx::query_as!(
        Installation,
        "INSERT INTO installations (id) VALUES ($1) ON CONFLICT (id) DO NOTHING RETURNING id, account_id",
        installation_id
    )
    .fetch_one(executor)
    .await
}