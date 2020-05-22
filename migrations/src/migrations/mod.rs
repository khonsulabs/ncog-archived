mod migration_0001_accounts;
use futures::executor::block_on;
use lazy_static::lazy_static;
use sqlx_simple_migrator::{Migration, MigrationError};
use std::env;

pub fn migrations() -> Vec<Migration> {
    vec![
        migration_0001_accounts::migration(),
    ]
}

use sqlx::PgPool;

lazy_static! {
    static ref POOL: PgPool = {
        block_on(PgPool::new(
            &env::var("DATABASE_URL").expect("DATABASE_URL not set"),
        ))
        .expect("Error initializing postgres pool")
    };
}

pub fn pg() -> PgPool {
    POOL.clone()
}

pub async fn run_all() -> Result<(), MigrationError> {
    let mut pool = pg();

    Migration::run_all(&mut pool, migrations()).await
}