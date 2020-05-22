use sqlx_simple_migrator::Migration;

pub fn migration() -> Migration {
    Migration::new(std::file!())
        .with_up(
            r#"
        CREATE TABLE accounts (
            id BIGSERIAL PRIMARY KEY,
            screenname TEXT NULL UNIQUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        "#,
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
        .with_down(
            r#"
        DROP TABLE IF EXISTS itchio_profiles
        "#,
        )
        .debug()
}
