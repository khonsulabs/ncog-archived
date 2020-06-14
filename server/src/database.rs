use shared::{
    iam::{RoleSummary, User},
    permissions::{PermissionSet, Statement},
    Installation, UserProfile,
};
use uuid::Uuid;

use migrations::sqlx;

use chrono::{DateTime, Utc};
use sqlx::executor::RefExecutor;
use sqlx::{postgres::Postgres, prelude::*};

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

pub async fn iam_list_users<'e, E>(executor: E) -> Result<Vec<User>, sqlx::Error>
where
    E: 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    let mut users: Vec<User> = Vec::new();

    // TODO https://github.com/launchbadge/sqlx/issues/367 Once this is shipping, we can switch this to strongly typed query again
    let mut user_rows = sqlx::query(r#"SELECT accounts.id, screenname, created_at, roles.id as role_id, roles.name as role_name FROM accounts 
            LEFT OUTER JOIN account_roles ON account_roles.account_id = accounts.id
            LEFT OUTER JOIN roles ON roles.id = account_roles.role_id ORDER BY accounts.id"#).fetch(executor);
    while let Some(row) = user_rows.next().await? {
        let id = row.get::<i64, _>(0);
        if users.len() == 0 || users[users.len() - 1].id != id {
            users.push(User {
                id,
                screenname: row.get::<Option<String>, _>(1),
                created_at: row.get::<DateTime<Utc>, _>(2),
                roles: Vec::new(),
            });
        }

        match row.get::<Option<i64>, _>(3) {
            Some(role_id) => {
                let role_name = row.get::<String, _>(4);
                let users_count = users.len();
                users
                    .get_mut(users_count - 1)
                    .unwrap()
                    .roles
                    .push(RoleSummary {
                        id: role_id,
                        name: role_name,
                    });
            }
            None => {}
        }
    }

    Ok(users)
}

pub async fn iam_get_user<'e, E>(executor: E, account_id: i64) -> Result<Option<User>, sqlx::Error>
where
    E: 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    let mut user = None;

    // TODO https://github.com/launchbadge/sqlx/issues/367 Once this is shipping, we can switch this to strongly typed query again
    let mut user_rows = sqlx::query(r#"SELECT accounts.id, screenname, created_at, roles.id as role_id, roles.name as role_name FROM accounts 
            LEFT OUTER JOIN account_roles ON account_roles.account_id = accounts.id
            LEFT OUTER JOIN roles ON roles.id = account_roles.role_id WHERE accounts.id = $1 ORDER BY accounts.id"#).bind(&account_id).fetch(executor);
    while let Some(row) = user_rows.next().await? {
        user = Some(match user {
            Some(user) => user,
            None => User {
                id: row.get::<i64, _>(0),
                screenname: row.get::<Option<String>, _>(1),
                created_at: row.get::<DateTime<Utc>, _>(2),
                roles: Vec::new(),
            },
        });

        match row.get::<Option<i64>, _>(3) {
            Some(role_id) => {
                let role_name = row.get::<String, _>(4);
                user.as_mut().unwrap().roles.push(RoleSummary {
                    id: role_id,
                    name: role_name,
                });
            }
            None => {}
        }
    }

    Ok(user)
}

pub async fn iam_list_roles<'e, E>(executor: E) -> Result<Vec<RoleSummary>, sqlx::Error>
where
    E: 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    sqlx::query_as!(RoleSummary, "SELECT id, name FROM roles")
        .fetch_all(executor)
        .await
}
