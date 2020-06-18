mod migration_0001_accounts;
mod migration_0002_reset_accounts;
use crate::connection::pg;
use sqlx_simple_migrator::{Migration, MigrationError};

const JONS_ACCOUNT_ID: i64 = 1;
const JONS_ITCHIO_ID: i64 = 1997167;
const TIMELORD_ROLE_ID: i64 = 1;

pub fn migrations() -> Vec<Migration> {
    vec![
        migration_0001_accounts::migration(),
        migration_0002_reset_accounts::migration(),
    ]
}

pub async fn run_all() -> Result<(), MigrationError> {
    let pool = pg();

    Migration::run_all(&pool, migrations()).await
}
