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

struct RolePermissionStatement {
    pub id: i64,
    pub role_id: Option<i64>,
    pub service: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<i64>,
    pub allow: bool,
}

impl RolePermissionStatement {
    pub fn score(&self) -> i32 {
        let mut score = 0i32;
        score += self.role_id.as_ref().map_or(0, |_| 1 << 5);
        score += self.service.as_ref().map_or(0, |_| 1 << 4);
        score += self.action.as_ref().map_or(0, |_| 1 << 3);
        score += self.resource_type.as_ref().map_or(0, |_| 1 << 2);
        score += self.resource_id.as_ref().map_or(0, |_| 1 << 1);

        score
    }
}

pub async fn check_permission<'e, E>(
    executor: E,
    account_id: i64,
    service: &str,
    resource_type: Option<&str>,
    resource_id: Option<i64>,
    action: &str,
) -> Result<bool, sqlx::Error>
where
    E: 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    let mut best_match: Option<RolePermissionStatement> = None;
    for statement in sqlx::query_as!(
        RolePermissionStatement,
        r#"SELECT role_permission_statements.id, roles.id as role_id, service, action, resource_type, resource_id, allow FROM role_permission_statements 
            LEFT OUTER JOIN roles ON role_permission_statements.role_id = roles.id 
            LEFT OUTER JOIN account_roles ON account_roles.role_id = roles.id
            LEFT OUTER JOIN accounts ON accounts.id = account_roles.account_id
            WHERE 
                (accounts.id IS NULL OR accounts.id = $1)
                AND (service IS NULL OR service = $2)
                AND (resource_type IS NULL OR resource_type = $4)
                AND (resource_id IS NULL or resource_id = $5)
                AND (action IS NULL OR action = $3)"#,
        account_id,
        service,
        action,
        resource_type,
        resource_id
    )
    .fetch_all(executor)
    .await?
    {
        best_match = Some(if let Some(best_match) = best_match {
            // Take whichever statement has the highest relevance score. If they match, take the allow statement if there is one.
            if statement.score() > best_match.score() || (statement.allow && statement.score() == best_match.score()) {
                statement
            } else {
                best_match
            }
        } else {
            statement
        });
    }
    Ok(best_match.map_or(false, |stmt| stmt.allow))
}
