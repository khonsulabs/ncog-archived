use sqlx_simple_migrator::Migration;

pub fn migration() -> Migration {
    Migration::new(std::file!())
        // Purposely ignoring not having a down to restore the table.
        .with_up("ALTER TABLE installations ADD COLUMN private_key BYTEA")
        .with_down("ALTER TABLE installations DROP COLUMN IF EXISTS private_key")
        .debug()
}
