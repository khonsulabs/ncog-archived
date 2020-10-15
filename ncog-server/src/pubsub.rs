use crate::{database, websockets::ConnectedAccount, websockets::NcogServer};
use basws_server::{Handle, Server};
use ncog_migrations::{pg, sqlx};
use ncog_shared::NcogResponse;
use sqlx::{executor::Executor, postgres::PgListener};
use std::collections::HashSet;
use uuid::Uuid;

pub async fn pg_notify_loop(websockets: Server<NcogServer>) -> Result<(), anyhow::Error> {
    let pool = pg();
    let mut listener = PgListener::from_pool(&pool).await?;
    listener
        .listen_all(vec!["installation_login", "world_update", "role_updated"])
        .await?;
    while let Ok(notification) = listener.recv().await {
        info!(
            "Got notification: {} {}",
            notification.channel(),
            notification.payload()
        );
        if notification.channel() == "installation_login" {
            // The payload is the installation_id that logged in.
            let installation_id = Uuid::parse_str(notification.payload())?;
            if let Ok(account) = ConnectedAccount::lookup(installation_id).await {
                let user = account.user.clone();
                websockets
                    .associate_installation_with_account(installation_id, Handle::new(account))
                    .await?;

                websockets
                    .send_to_installation_id(installation_id, NcogResponse::Authenticated(user))
                    .await;
            }
        } else if notification.channel() == "role_updated" {
            let role_id = notification.payload().parse::<i64>()?;
            let mut refreshed_accounts = HashSet::new();
            for client in websockets.connected_clients().await {
                if let Some(account) = client.account().await {
                    let mut account = account.write().await;
                    if !refreshed_accounts.contains(&account.user.profile.id)
                        && account.user.permissions.role_ids.contains(&role_id)
                    {
                        refreshed_accounts.insert(account.user.profile.id);
                        account.user.permissions =
                            database::load_permissions_for(&pg(), account.user.profile.id).await?;
                        websockets
                            .send_to_account_id(
                                account.user.profile.id,
                                NcogResponse::Authenticated(account.user.clone()),
                            )
                            .await;
                    }
                }
            }
        }
    }
    panic!("Error on postgres listening");
}

pub async fn notify<S: ToString>(channel: &'static str, payload: S) -> Result<(), sqlx::Error> {
    let mut connection = pg().acquire().await?;
    connection
        .execute(&*format!("NOTIFY {}, '{}'", channel, payload.to_string()))
        .await?;
    Ok(())
}
