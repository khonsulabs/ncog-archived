mod migration_0001_accounts;
use sqlx_simple_migrator::{Migration, MigrationError};
use crate::connection::pg;

pub fn migrations() -> Vec<Migration> {
    vec![
        migration_0001_accounts::migration(),
    ]
}

pub async fn run_all() -> Result<(), MigrationError> {
    let mut pool = pg();

    Migration::run_all(&mut pool, migrations()).await
}