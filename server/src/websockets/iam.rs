use crate::{
    database,
    websockets::{ConnectedAccountHandle, ConnectedClient},
};
use basws_server::RequestHandling;
use migrations::pg;
use shared::{
    iam::{
        roles_delete_claim, roles_list_claim, roles_read_claim, roles_update_claim,
        users_list_claim, users_read_claim, IAMRequest, IAMResponse,
    },
    ServerResponse,
};

pub async fn handle_request(
    client_handle: &ConnectedClient<super::NcogServer>,
    request: IAMRequest,
) -> anyhow::Result<RequestHandling<ServerResponse>> {
    match request {
        IAMRequest::UsersList => {
            client_handle
                .permission_allowed(&users_list_claim())
                .await?;

            let mut users = Vec::new();

            for user in database::iam_list_users(&pg()).await? {
                if client_handle
                    .permission_allowed(&users_read_claim(user.id))
                    .await
                    .is_ok()
                {
                    users.push(user);
                }
            }

            Ok(RequestHandling::Respond(ServerResponse::IAM(
                IAMResponse::UsersList(users),
            )))
        }
        IAMRequest::UsersGetProfile(account_id) => {
            client_handle
                .permission_allowed(&users_read_claim(Some(account_id)))
                .await?;

            let user = database::iam_get_user(&pg(), account_id).await?;

            match user {
                Some(user) => Ok(RequestHandling::Respond(ServerResponse::IAM(
                    IAMResponse::UserProfile(user),
                ))),
                None => anyhow::bail!("Unknown user id {}", account_id),
            }
        }
        IAMRequest::RolesList => {
            client_handle
                .permission_allowed(&roles_list_claim())
                .await?;

            let mut roles = Vec::new();

            for role in database::iam_list_roles(&pg()).await? {
                if client_handle
                    .permission_allowed(&roles_read_claim(role.id))
                    .await
                    .is_ok()
                {
                    roles.push(role);
                }
            }

            Ok(RequestHandling::Respond(ServerResponse::IAM(
                IAMResponse::RolesList(roles),
            )))
        }
        IAMRequest::RoleGet(role_id) => {
            client_handle
                .permission_allowed(&roles_read_claim(Some(role_id)))
                .await?;

            let role = database::iam_get_role(&pg(), role_id).await?;

            match role {
                Some(role) => Ok(RequestHandling::Respond(ServerResponse::IAM(
                    IAMResponse::Role(role),
                ))),
                None => anyhow::bail!("Unknown role id {}", role_id),
            }
        }
        IAMRequest::RoleSave(role) => {
            client_handle
                .permission_allowed(&roles_update_claim(role.id))
                .await?;

            let role_id = database::iam_update_role(&pg(), &role).await?;

            Ok(RequestHandling::Respond(ServerResponse::IAM(
                IAMResponse::RoleSaved(role_id),
            )))
        }
        IAMRequest::RoleDelete(role_id) => {
            client_handle
                .permission_allowed(&roles_delete_claim(Some(role_id)))
                .await?;

            let mut tx = pg().begin().await?;
            database::iam_delete_role(&mut tx, role_id).await?;

            Ok(RequestHandling::Respond(ServerResponse::IAM(
                IAMResponse::RoleDeleted(role_id),
            )))
        }
        IAMRequest::PermissionStatementGet(id) => {
            let statement = database::iam_get_permission_statement(&pg(), id).await?;
            client_handle
                .permission_allowed(&roles_read_claim(statement.role_id))
                .await?;

            Ok(RequestHandling::Respond(ServerResponse::IAM(
                IAMResponse::PermissionStatement(statement),
            )))
        }
        IAMRequest::PermissionStatementSave(statement) => {
            // TODO Validate that the user can edit the currently assigned role

            client_handle
                .permission_allowed(&roles_update_claim(statement.role_id))
                .await?;

            let statement_id = database::iam_update_permission_statement(&pg(), &statement).await?;

            broadcast_role_changed(statement.role_id).await?;

            Ok(RequestHandling::Respond(ServerResponse::IAM(
                IAMResponse::PermissionStatementSaved(statement_id),
            )))
        }
        IAMRequest::PermissionStatemenetDelete(id) => {
            let statement = database::iam_get_permission_statement(&pg(), id).await?;

            client_handle
                .permission_allowed(&roles_update_claim(statement.role_id))
                .await?;

            database::iam_delete_permission_statement(&pg(), id).await?;

            broadcast_role_changed(statement.role_id).await?;

            Ok(RequestHandling::Respond(ServerResponse::IAM(
                IAMResponse::PermissionStatementDeleted(id),
            )))
        }
    }
}

async fn broadcast_role_changed(role_id: Option<i64>) -> Result<(), anyhow::Error> {
    if let Some(role_id) = role_id {
        crate::pubsub::notify("role_updated", role_id).await?;
    }
    Ok(())
}
