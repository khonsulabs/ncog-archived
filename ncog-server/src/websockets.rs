use super::{database, env, twitch};
use async_trait::async_trait;
use ncog_migrations::{pg, sqlx};
use ncog_shared::{
    ncog_protocol_version_requirements,
    permissions::{Claim, PermissionSet},
    NcogRequest, NcogResponse, OAuthProvider, UserProfile,
};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
mod iam;
use basws_server::prelude::*;

#[async_trait]
pub trait ConnectedAccountHandle {
    async fn permission_allowed(&self, claim: &Claim) -> Result<(), anyhow::Error>;
}

fn permission_denied(claim: &Claim) -> Result<(), anyhow::Error> {
    anyhow::bail!("permission denied for accessing {:?}", claim)
}

#[async_trait]
impl ConnectedAccountHandle for ConnectedClient<NcogServer> {
    async fn permission_allowed(&self, claim: &Claim) -> Result<(), anyhow::Error> {
        if let Some(account) = self.account().await {
            return account.permission_allowed(claim).await;
        }

        permission_denied(claim)
    }
}

#[async_trait]
impl ConnectedAccountHandle for Handle<ConnectedAccount> {
    async fn permission_allowed(&self, claim: &Claim) -> Result<(), anyhow::Error> {
        let account = self.read().await;
        if account.permissions.allowed(&claim) {
            Ok(())
        } else {
            permission_denied(claim)
        }
    }
}

#[derive(Debug)]
pub struct ConnectedAccount {
    pub profile: UserProfile,
    pub permissions: PermissionSet,
}

impl ConnectedAccount {
    pub async fn lookup(installation_id: Uuid) -> anyhow::Result<Self> {
        let profile = database::get_profile_by_installation_id(&pg(), installation_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("no profile found"))?;
        let permissions = database::load_permissions_for(&pg(), profile.id).await?;
        Ok(Self {
            profile,
            permissions,
        })
    }
}

impl Identifiable for ConnectedAccount {
    type Id = i64;
    fn id(&self) -> Self::Id {
        self.profile.id
    }
}
pub struct NcogServer;

pub fn initialize() -> Server<NcogServer> {
    Server::new(NcogServer)
}

#[async_trait]
impl ServerLogic for NcogServer {
    type Request = NcogRequest;
    type Response = NcogResponse;
    type Client = ();
    type Account = ConnectedAccount;
    type AccountId = i64;

    async fn handle_request(
        &self,
        client: &ConnectedClient<Self>,
        request: Self::Request,
        _server: &Server<Self>,
    ) -> anyhow::Result<RequestHandling<Self::Response>> {
        match request {
            NcogRequest::AuthenticationUrl(provider) => match provider {
                OAuthProvider::Twitch => {
                    if let Some(installation) = client.installation().await {
                        Ok(RequestHandling::Respond(NcogResponse::AuthenticateAtUrl {
                            url: twitch::authorization_url(installation.id),
                        }))
                    } else {
                        anyhow::bail!("Requested authentication URL without being connected")
                    }
                }
            },
            NcogRequest::IAM(iam_request) => iam::handle_request(client, iam_request).await,
        }
    }

    async fn lookup_account_from_installation_id(
        &self,
        installation_id: Uuid,
    ) -> anyhow::Result<Option<Handle<Self::Account>>> {
        let account = ConnectedAccount::lookup(installation_id).await?;
        Ok(Some(Handle::new(account)))
    }

    fn protocol_version_requirements(&self) -> VersionReq {
        ncog_protocol_version_requirements()
    }

    async fn lookup_or_create_installation(
        &self,
        _client: &ConnectedClient<Self>,
        installation_id: Option<Uuid>,
    ) -> anyhow::Result<InstallationConfig> {
        let installation = database::lookup_or_create_installation(installation_id).await?;
        Ok(InstallationConfig::from_vec(
            installation.id,
            installation.private_key.unwrap(),
        )?)
    }

    async fn client_reconnected(
        &self,
        client: &ConnectedClient<Self>,
    ) -> anyhow::Result<RequestHandling<Self::Response>> {
        if let Some(account) = client.account().await {
            let account = account.read().await;

            Ok(RequestHandling::Respond(NcogResponse::Authenticated {
                profile: account.profile.clone(),
                permissions: account.permissions.clone(),
            }))
        } else {
            Ok(RequestHandling::Respond(NcogResponse::Unauthenticated))
        }
    }

    async fn new_client_connected(
        &self,
        _client: &ConnectedClient<Self>,
    ) -> anyhow::Result<RequestHandling<Self::Response>> {
        Ok(RequestHandling::Respond(NcogResponse::Unauthenticated))
    }

    async fn account_associated(&self, client: &ConnectedClient<Self>) -> anyhow::Result<()> {
        if let Some(installation) = client.installation().await {
            if let Some(account) = client.account().await {
                let account_id = {
                    let account = account.read().await;
                    account.id()
                };
                database::set_installation_account_id(&pg(), installation.id, Some(account_id))
                    .await?;
                return Ok(());
            }
        }
        anyhow::bail!("account_associated called with either no installation or account")
    }

    async fn handle_websocket_error(&self, _err: warp::Error) -> ErrorHandling {
        ErrorHandling::Disconnect
    }

    async fn client_disconnected(&self, client: &ConnectedClient<Self>) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ItchioProfile {
    pub cover_url: Option<String>,
    pub display_name: Option<String>,
    pub username: String,
    pub id: i64,
    pub developer: bool,
    pub gamer: bool,
    pub press_user: bool,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TwitchTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<usize>,
    pub scope: Vec<String>,
    pub id_token: String,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TwitchUserInfo {
    pub id: String,
    pub login: String,
    pub display_name: Option<String>,
    // pub type: Option<String>,
    // pub broadcaster_type: String,
    // pub description: Option<String>,
    // pub profile_image_url: Option<String>,
    // pub offline_image_url: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct TwitchUsersResponse {
    pub data: Vec<TwitchUserInfo>,
}

pub async fn login_twitch(installation_id: Uuid, code: String) -> Result<(), anyhow::Error> {
    // Call itch.io API to get the user information
    let client = reqwest::Client::new();
    let tokens: TwitchTokenResponse = client
        .post("https://id.twitch.tv/oauth2/token")
        .query(&[
            ("code", code),
            ("client_id", env("TWITCH_CLIENT_ID")),
            ("client_secret", env("TWITCH_CLIENT_SECRET")),
            ("grant_type", "authorization_code".to_owned()),
            ("redirect_uri", twitch::callback_uri()),
        ])
        .send()
        .await?
        .json()
        .await?;

    // TODO validate the id_token https://dev.twitch.tv/docs/authentication/getting-tokens-oidc

    let response: TwitchUsersResponse = client
        .get("https://api.twitch.tv/helix/users")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", tokens.access_token),
        )
        .header("client-id", env("TWITCH_CLIENT_ID"))
        .send()
        .await?
        .json()
        .await?;

    let user = response
        .data
        .first()
        .ok_or_else(|| anyhow::anyhow!("Expected a user response, but got no users"))?;

    let display_name = user
        .display_name
        .clone()
        .unwrap_or_else(|| user.login.clone());
    let pg = pg();
    {
        let mut tx = pg.begin().await?;

        // Create an account if it doesn't exist yet for this installation
        let account_id = if let Some(account) =
            database::get_profile_by_installation_id(&mut tx, installation_id).await?
        {
            if account.login.is_none() {
                // If this generates a conflict, ignore it.
                let _ = sqlx::query!(
                    "UPDATE accounts SET login = $1, display_name = $2 WHERE id = $3",
                    user.login,
                    display_name,
                    account.id
                )
                .execute(&mut tx)
                .await?;
            }
            account.id
        } else {
            let account_id = if let Ok(row) = sqlx::query!(
                "SELECT account_id, login FROM twitch_profiles INNER JOIN accounts ON accounts.id = account_id WHERE twitch_profiles.id = $1",
                user.id
            )
            .fetch_one(&mut tx)
            .await
            {
                if row.login.is_none() {
                    let _ = sqlx::query!(
                        "UPDATE accounts SET login = $1, display_name = $2 WHERE id = $3",
                        user.login,
                        display_name,
                        row.account_id
                    )
                    .execute(&mut tx)
                    .await?;
                }
                row.account_id
            } else {
                sqlx::query!(
                    "INSERT INTO accounts (login, display_name) VALUES ($1, $2) RETURNING id",
                    user.login,
                    display_name
                )
                .fetch_one(&mut tx)
                .await?
                .id
            };
            database::set_installation_account_id(&mut tx, installation_id, Some(account_id))
                .await?;
            account_id
        };

        // Create an twitch profile
        sqlx::query!("INSERT INTO twitch_profiles (id, account_id, username) VALUES ($1, $2, $3) ON CONFLICT (id) DO UPDATE SET account_id = $2, username = $3 ",
            user.id,
            account_id,
            display_name,
        ).execute(&mut tx).await?;

        // Create an oauth_token
        sqlx::query!("INSERT INTO oauth_tokens (account_id, service, access_token, refresh_token) VALUES ($1, $2, $3, $4) ON CONFLICT (account_id, service) DO UPDATE SET access_token = $3, refresh_token = $4",
            account_id,
            "twitch",
            tokens.access_token,
            tokens.refresh_token,

        ).execute(&mut tx).await?;

        tx.commit().await?;
    }

    crate::pubsub::notify("installation_login", installation_id.to_string()).await?;

    Ok(())
}
