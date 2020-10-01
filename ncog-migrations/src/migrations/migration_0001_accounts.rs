use super::{JONS_ACCOUNT_ID, JONS_ITCHIO_ID, TIMELORD_ROLE_ID};
use sqlx_simple_migrator::Migration;

pub fn migration() -> Migration {
    Migration::new("0001")
        .with_up(
            r#"
        CREATE TABLE accounts (
            id BIGSERIAL PRIMARY KEY,
            screenname TEXT NULL UNIQUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        "#,
        )
        .with_up(
            "INSERT INTO accounts DEFAULT VALUES"
        )
        .with_down(
            r#"
            DROP TABLE IF EXISTS accounts
        "#,
        )
        .with_up(
            r#"
        CREATE TABLE installations (
            id UUID PRIMARY KEY,
            account_id BIGINT NULL REFERENCES accounts(id)
        )
        "#,
        )
        .with_down(
            r#"
        DROP TABLE IF EXISTS installations
        "#,
        )
        .with_up(
            r#"
        CREATE TABLE oauth_tokens (
            account_id BIGINT NOT NULL REFERENCES accounts(id),
            service TEXT NOT NULL,
            refresh_token TEXT,
            access_token TEXT NOT NULL,
            expires TIMESTAMP NULL,
            PRIMARY KEY (service, account_id)
        )
        "#,
        )
        .with_down(
            r#"
        DROP TABLE IF EXISTS oauth_tokens
        "#,
        )
        .with_up(
            r#"
        CREATE TABLE itchio_profiles (
            id BIGINT PRIMARY KEY,
            account_id BIGINT NOT NULL REFERENCES accounts(id),
            username TEXT NOT NULL,
            url TEXT
        )
        "#,
        )
        .with_up(&format!(
            "INSERT INTO itchio_profiles (id, account_id, username, url) values ({}, {}, 'khonsulabs', '')",
            JONS_ITCHIO_ID,
            JONS_ACCOUNT_ID
        ))
        .with_down(
            r#"
        DROP TABLE IF EXISTS itchio_profiles
        "#,
        )
        .with_up(
            r#"
        CREATE TABLE roles (
            id BIGSERIAL PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        )
        "#,
        )
        .with_up(
            "INSERT INTO roles (name) values ('Time Lord')"
        )
        .with_down(
            r#"
        DROP TABLE IF EXISTS roles
        "#,
        )
        .with_up(
            r#"
        CREATE TABLE role_permission_statements (
            id BIGSERIAL PRIMARY KEY,
            role_id BIGINT NULL REFERENCES roles(id),
            action TEXT NULL,
            service TEXT NULL,
            resource_type TEXT NULL,
            resource_id BIGINT NULL,
            allow BOOL NOT NULL,
            comment TEXT NULL,
            CHECK (resource_type IS NOT NULL OR resource_id IS NULL)
        )
        "#,
        )
        .with_up(&format!(
            "INSERT INTO role_permission_statements (role_id, allow) values ({}, true)",
            TIMELORD_ROLE_ID
        ))
        .with_up(
            "INSERT INTO role_permission_statements (service, action, allow) values ('ncog', 'connect', true)"
        )
        .with_down(
            r#"
        DROP TABLE IF EXISTS role_permission_statements
        "#,
        )
        .with_up(
            r#"
        CREATE TABLE account_roles (
            role_id BIGINT NOT NULL REFERENCES roles(id),
            account_id BIGINT NOT NULL REFERENCES accounts(id),
            PRIMARY KEY (account_id, role_id)
        )
        "#,
        )
        .with_up(&format!(
            "INSERT INTO account_roles (account_id, role_id) values ({}, {})",
            JONS_ACCOUNT_ID,
            TIMELORD_ROLE_ID
        ))
        .with_down(
            r#"
        DROP TABLE IF EXISTS account_roles
        "#,
        )
}
