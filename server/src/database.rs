use shared::{
    iam::{PermissionStatement, Role, RoleSummary, User},
    permissions::{PermissionSet, Statement},
    Installation, UserProfile,
};
use uuid::Uuid;

use migrations::sqlx;

use chrono::{DateTime, Utc};
use sqlx::executor::RefExecutor;
use sqlx::{postgres::Postgres, prelude::*};

pub async fn get_profile_by_installation_id<'e, E>(
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

pub async fn get_profile_by_account_id<'e, E>(
    executor: E,
    account_id: i64,
) -> Result<UserProfile, sqlx::Error>
where
    E: 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        UserProfile,
        "SELECT accounts.id, screenname FROM accounts WHERE accounts.id = $1",
        account_id,
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
    let results  = sqlx::query_as!(Statement, r#"SELECT roles.id as role_id, service, resource_type, resource_id, action, allow FROM role_permission_statements 
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
        let id = row.get::<Option<i64>, _>(0);
        if users.is_empty() || users[users.len() - 1].id != id {
            users.push(User {
                id,
                screenname: row.get::<Option<String>, _>(1),
                created_at: row.get::<DateTime<Utc>, _>(2),
                roles: Vec::new(),
            });
        }

        if let Some(role_id) = row.get::<Option<i64>, _>(3) {
            let role_name = row.get::<String, _>(4);
            let users_count = users.len();
            users
                .get_mut(users_count - 1)
                .unwrap()
                .roles
                .push(RoleSummary {
                    id: Some(role_id),
                    name: role_name,
                });
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
                id: row.get::<Option<i64>, _>(0),
                screenname: row.get::<Option<String>, _>(1),
                created_at: row.get::<DateTime<Utc>, _>(2),
                roles: Vec::new(),
            },
        });

        if let Some(role_id) = row.get::<Option<i64>, _>(3) {
            let role_name = row.get::<String, _>(4);
            user.as_mut().unwrap().roles.push(RoleSummary {
                id: Some(role_id),
                name: role_name,
            });
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

pub async fn iam_get_role<'e, E>(executor: E, role_id: i64) -> Result<Option<Role>, sqlx::Error>
where
    E: Copy + 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    let summary = match sqlx::query_as!(
        RoleSummary,
        "SELECT id, name FROM roles WHERE id = $1",
        role_id
    )
    .fetch_one(executor)
    .await
    {
        Ok(role) => role,
        Err(err) => match err {
            sqlx::Error::RowNotFound => return Ok(None),
            _ => return Err(err),
        },
    };

    let permission_statements = sqlx::query_as!(
        PermissionStatement,
        r#"SELECT *
        FROM role_permission_statements
        WHERE role_id = $1
        ORDER BY id"#,
        role_id
    )
    .fetch_all(executor)
    .await?;

    Ok(Some(Role {
        id: summary.id,
        name: summary.name,
        permission_statements,
    }))
}

pub async fn iam_update_role<'e, E>(executor: E, role: &RoleSummary) -> Result<i64, sqlx::Error>
where
    E: 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    let id = sqlx::query!(
        r#"INSERT INTO roles (id, name) VALUES (COALESCE($1, (SELECT nextval('roles_id_seq'))), $2) 
            ON CONFLICT (id) DO UPDATE SET name = $2
            RETURNING id"#,
        role.id,
        &role.name
    )
    .fetch_one(executor)
    .await?
    .id;

    Ok(id)
}

pub async fn iam_get_permission_statement<'e, E>(
    executor: E,
    permission_statement_id: i64,
) -> Result<PermissionStatement, sqlx::Error>
where
    E: Copy + 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        PermissionStatement,
        r#"SELECT *
        FROM role_permission_statements
        WHERE id = $1"#,
        permission_statement_id
    )
    .fetch_one(executor)
    .await
}

pub async fn iam_update_permission_statement<'e, E>(
    executor: E,
    statement: &PermissionStatement,
) -> Result<i64, sqlx::Error>
where
    E: 'e + Send + RefExecutor<'e, Database = Postgres>,
{
    let id = sqlx::query(
        r#"INSERT INTO role_permission_statements (
                id, 
                role_id, 
                service, 
                resource_type, 
                resource_id, 
                action, 
                allow, 
                comment
            ) VALUES (COALESCE($1, (SELECT nextval('role_permission_statements_id_seq'))), $2, $3, $4, $5, $6, $7, $8) 
            ON CONFLICT (id) DO UPDATE SET role_id = $2, service = $3, resource_type = $4, resource_id = $5, action = $6, allow = $7, comment = $8
            RETURNING id"#)
        .bind(statement.id)
        .bind(statement.role_id)
        .bind(&statement.service)
        .bind(&statement.resource_type)
        .bind(statement.resource_id)
        .bind(&statement.action)
        .bind(statement.allow)
        .bind(&statement.comment)
    
    .fetch(executor).next().await?.expect("Error saving permission statement").get::<i64,_>(0);

    Ok(id)
}


pub async fn iam_delete_permission_statement<E>(
    executor: E,
    id: i64
) -> Result<(), sqlx::Error>
where
    E: Send + Executor<Database = Postgres>,
{
    sqlx::query!(
        "DELETE FROM role_permission_statements WHERE id=$1", id)
        .execute(executor).await?;

    Ok(())
}