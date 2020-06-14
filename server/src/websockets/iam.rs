use crate::{
    database,
    websockets::{ConnectedAccountHandle, ConnectedClient},
};
use async_std::sync::RwLock;
use shared::{
    iam::{IAMRequest, IAMResponse},
    permissions::Claim,
    websockets::WsBatchResponse,
    ServerResponse,
};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub async fn handle_request(
    client_handle: &Arc<RwLock<ConnectedClient>>,
    request: IAMRequest,
    responder: UnboundedSender<WsBatchResponse>,
    request_id: i64,
) -> Result<(), anyhow::Error> {
    match request {
        IAMRequest::UsersList => {
            client_handle
                .permission_allowed(&Claim::new("iam", Some("users"), None, "list"))
                .await?;

            let mut users = Vec::new();

            for user in database::iam_list_users(&migrations::pg()).await? {
                if client_handle
                    .permission_allowed(&Claim::new("iam", Some("users"), Some(user.id), "read"))
                    .await
                    .is_ok()
                {
                    users.push(user);
                }
            }

            responder.send(
                ServerResponse::IAM(IAMResponse::UsersList(users)).into_ws_response(request_id),
            )?;
        }
        IAMRequest::UsersGetProfile(account_id) => {
            client_handle
                .permission_allowed(&Claim::new("iam", Some("users"), Some(account_id), "read"))
                .await?;

            let user = database::iam_get_user(&migrations::pg(), account_id).await?;

            match user {
                Some(user) => {
                    responder.send(
                        ServerResponse::IAM(IAMResponse::UserProfile(user))
                            .into_ws_response(request_id),
                    )?;
                }
                None => anyhow::bail!("Unknown user id {}", account_id),
            }
        }
    }

    Ok(())
}
