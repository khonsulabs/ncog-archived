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

            let users = database::iam_list_users(&migrations::pg()).await?;

            responder.send(
                ServerResponse::IAM(IAMResponse::UsersList(users)).into_ws_response(request_id),
            )?;
        }
    }

    Ok(())
}
