use super::{JONS_ACCOUNT_ID, JONS_TWITCH_ID};
use sqlx_simple_migrator::Migration;

pub fn migration() -> Migration {
    Migration::new(&format!(
        "migrations{}{}",
        std::path::MAIN_SEPARATOR,
        std::file!()
    ))
    // Purposely ignoring not having a down to restore the table.
    .with_up(
        r#"
        DROP TABLE IF EXISTS itchio_profiles CASCADE;
        "#,
    )
    .with_up(
        r#"
        CREATE TABLE twitch_profiles (
            id TEXT PRIMARY KEY,
            account_id BIGINT NOT NULL REFERENCES accounts(id),
            username TEXT NOT NULL
        )
        "#,
    )
    .with_up(&format!(
        "INSERT INTO twitch_profiles (id, account_id, username) values ({}, {}, 'ectondev')",
        JONS_TWITCH_ID, JONS_ACCOUNT_ID
    ))
    .with_down("DROP TABLE IF EXISTS twitch_profiles")
    .with_up("ALTER TABLE installations ADD COLUMN nonce BYTEA")
    .with_down("ALTER TABLE installations DROP COLUMN IF EXISTS nonce")
}
