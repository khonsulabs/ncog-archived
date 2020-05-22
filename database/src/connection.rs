use futures::executor::block_on;
use lazy_static::lazy_static;
use sqlx::PgPool;
use std::env;

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
