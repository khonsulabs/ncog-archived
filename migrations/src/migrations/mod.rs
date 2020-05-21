mod migration_0001_accounts;
mod migration_0002_movement;
use futures::executor::block_on;
use lazy_static::lazy_static;
use sqlx_simple_migrator::{Migration, MigrationError};
use std::env;

pub fn migrations() -> Vec<Migration> {
    vec![
        migration_0001_accounts::migration(),
        migration_0002_movement::migration(),
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

#[cfg(test)]
mod tests {
    use super::pg;
    use shared::Installation;
    use uuid::Uuid;
    #[tokio::test]
    async fn accounts_test() -> Result<(), sqlx::Error> {
        dotenv::dotenv().unwrap();
        let pool = pg();
        let mut tx = pool.begin().await?;

        // Create an installation
        let installation_id = Uuid::new_v4();
        let installation = sqlx::query_as!(
            Installation,
            "SELECT * FROM installation_lookup($1)",
            installation_id
        )
        .fetch_one(&mut tx)
        .await?;
        assert_eq!(installation_id, installation.id);
        assert_eq!(None, installation.account_id);

        // Simulate looking up an account for a user
        let account = sqlx::query!("SELECT account_lookup($1, $2) as account_id", 1, "username")
            .fetch_one(&mut tx)
            .await
            .expect("Function should always return a value");

        let repeated_account_lookup =
            sqlx::query!("SELECT account_lookup($1, $2) as account_id", 1, "username")
                .fetch_one(&mut tx)
                .await
                .expect("Function should always return a value");
        assert_eq!(repeated_account_lookup.account_id, account.account_id);

        // Assign the installation to the account
        let installation_set_account_result = sqlx::query!(
            "SELECT installation_login($1, $2, $3) as rows_changed",
            installation_id,
            account.account_id,
            "itchio_token"
        )
        .fetch_one(&mut tx)
        .await?;
        assert_eq!(installation_set_account_result.rows_changed, Some(1));

        // Check that the token is present when looking it up
        let account_token_result = sqlx::query!(
            "SELECT account_get_itchio_token($1) as token",
            account.account_id
        )
        .fetch_one(&mut tx)
        .await?;
        assert_eq!(account_token_result.token, Some("itchio_token".to_owned()));

        // transaction is automatically rolled back
        Ok(())
    }
}
