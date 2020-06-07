use crate::permissions::{PermissionSet, Statement};
use shared::{Installation, UserProfile};
use uuid::Uuid;

use migrations::sqlx;

use sqlx::executor::RefExecutor;
use sqlx::postgres::Postgres;

pub async fn get_profile<'e, E>(
    executor: E,
    installation_id: Uuid,
) -> Result<UserProfile, sqlx::Error>
where
    E: 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        UserProfile,
        "SELECT accounts.id, screenname FROM accounts INNER JOIN installations ON installations.account_id = accounts.id WHERE installations.id = $1",
        installation_id,
    )
    .fetch_one(executor)
    .await
}

pub async fn lookup_installation<'e, E>(
    executor: E,
    installation_id: Uuid,
) -> Result<Installation, sqlx::Error>
where
    E: 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        Installation,
        "INSERT INTO installations (id) VALUES ($1) ON CONFLICT (id) DO UPDATE SET id=$1 RETURNING id, account_id",
        installation_id
    )
    .fetch_one(executor)
    .await
}

pub async fn load_permissions_for<'e, E>(
    executor: E,
    account_id: i64,
) -> Result<PermissionSet, sqlx::Error>
where
    E: 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    let results  = sqlx::query_as!(Statement, r#"SELECT service, resource_type, resource_id, action, allow FROM role_permission_statements 
            LEFT OUTER JOIN roles ON role_permission_statements.role_id = roles.id
            LEFT OUTER JOIN account_roles ON account_roles.role_id = roles.id
            LEFT OUTER JOIN accounts ON accounts.id = account_roles.account_id
            WHERE 
                (accounts.id IS NULL OR accounts.id = $1)
        "#, account_id).fetch_all(executor).await?;

    Ok(results.into())
}
