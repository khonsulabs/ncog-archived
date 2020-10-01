use sqlx_simple_migrator::Migration;

pub fn migration() -> Migration {
    Migration::new(&format!(
        "migrations{}{}",
        std::path::MAIN_SEPARATOR,
        std::file!()
    ))
    .with_up("ALTER TABLE installations ADD COLUMN private_key BYTEA")
    .with_down("ALTER TABLE installations DROP COLUMN IF EXISTS private_key")
}
