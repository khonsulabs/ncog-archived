use crate::{
    database,
    websockets::{ConnectedAccountHandle, ConnectedClient},
};
use async_std::sync::RwLock;
use shared::{
    iam::{
        roles_list_claim, roles_read_claim, roles_update_claim, users_list_claim, users_read_claim,
        IAMRequest, IAMResponse,
    },
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
                .permission_allowed(&users_list_claim())
                .await?;

            let mut users = Vec::new();

            for user in database::iam_list_users(&migrations::pg()).await? {
                if client_handle
                    .permission_allowed(&users_read_claim(user.id))
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
                .permission_allowed(&users_read_claim(Some(account_id)))
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
        IAMRequest::RolesList => {
            client_handle
                .permission_allowed(&roles_list_claim())
                .await?;

            let mut roles = Vec::new();

            for role in database::iam_list_roles(&migrations::pg()).await? {
                if client_handle
                    .permission_allowed(&roles_read_claim(role.id))
                    .await
                    .is_ok()
                {
                    roles.push(role);
                }
            }

            responder.send(
                ServerResponse::IAM(IAMResponse::RolesList(roles)).into_ws_response(request_id),
            )?;
        }
        IAMRequest::RoleGet(role_id) => {
            client_handle
                .permission_allowed(&roles_read_claim(Some(role_id)))
                .await?;

            let role = database::iam_get_role(&migrations::pg(), role_id).await?;

            match role {
                Some(role) => {
                    responder.send(
                        ServerResponse::IAM(IAMResponse::Role(role)).into_ws_response(request_id),
                    )?;
                }
                None => anyhow::bail!("Unknown role id {}", role_id),
            }
        }
        IAMRequest::RoleSave(role) => {
            client_handle
                .permission_allowed(&roles_update_claim(role.id))
                .await?;

            let role_id = database::iam_update_role(&migrations::pg(), &role).await?;

            responder.send(
                ServerResponse::IAM(IAMResponse::RoleSaved(role_id)).into_ws_response(request_id),
            )?;
        }
        IAMRequest::PermissionStatementGet(id) => {
            let statement = database::iam_get_permission_statement(&migrations::pg(), id).await?;
            client_handle
                .permission_allowed(&roles_read_claim(statement.role_id))
                .await?;
            responder.send(
                ServerResponse::IAM(IAMResponse::PermissionStatement(statement))
                    .into_ws_response(request_id),
            )?;
        }
        IAMRequest::PermissionStatementSave(statement) => {
            client_handle
                .permission_allowed(&roles_update_claim(statement.role_id))
                .await?;

            let statement_id =
                database::iam_update_permission_statement(&migrations::pg(), &statement).await?;

            responder.send(
                ServerResponse::IAM(IAMResponse::PermissionStatementSaved(statement_id))
                    .into_ws_response(request_id),
            )?;
        }
    }

    Ok(())
}
