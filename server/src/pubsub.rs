use super::websockets::{CONNECTED_ACCOUNTS, CONNECTED_CLIENTS};
use migrations::{pg, sqlx};
use shared::ServerResponse;
use sqlx::postgres::PgListener;
use uuid::Uuid;

pub async fn pg_notify_loop() -> Result<(), anyhow::Error> {
    let pool = pg();
    let mut listener = PgListener::from_pool(&pool).await?;
    listener
        .listen_all(vec!["installation_login", "world_update"])
        .await?;
    while let Ok(notification) = listener.recv().await {
        println!(
            "Got notification: {} {}",
            notification.channel(),
            notification.payload()
        );
        if notification.channel() == "installation_login" {
            // The payload is the installation_id that logged in.
            let installation_id = Uuid::parse_str(notification.payload())?;
            let account = CONNECTED_ACCOUNTS.connect(installation_id).await?;
            let account = account.read().await;

            CONNECTED_CLIENTS
                .associate_account(installation_id, account.profile.id)
                .await?;

            CONNECTED_CLIENTS
                .send_to_installation_id(
                    installation_id,
                    ServerResponse::Authenticated {
                        profile: account.profile.clone(),
                    },
                )
                .await;
        } else if notification.channel() == "world_update" {
            // The payload is the timestamp of when the world was updated
            let timestamp = notification.payload().parse::<f64>()?;

            CONNECTED_CLIENTS.world_updated(timestamp).await?;
        }
    }
    panic!("Error on postgres listening");
}
