use super::{database, env, twitch};
use async_std::sync::RwLock;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use migrations::{pg, sqlx};
use serde_derive::{Deserialize, Serialize};
use shared::{
    current_timestamp,
    permissions::{Claim, PermissionSet},
    websockets::{WsBatchResponse, WsRequest},
    Inputs, OAuthProvider, ServerRequest, ServerResponse, UserProfile,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use uuid::Uuid;
use warp::filters::ws::{Message, WebSocket};
mod iam;

#[derive(Default)]
pub struct NetworkTiming {
    pub average_roundtrip: Option<f64>,
    pub average_server_timestamp_delta: Option<f64>,
}

impl NetworkTiming {
    pub fn update(&mut self, original_timestamp: f64, timestamp: f64) {
        let now = current_timestamp();
        let roundtrip = now - original_timestamp;

        self.average_roundtrip = Some(match self.average_roundtrip {
            Some(average_roundtrip) => (average_roundtrip * 4.0 + roundtrip) / 5.0,
            None => roundtrip,
        });

        let timestamp_delta = (now - timestamp) - roundtrip / 2.0;
        self.average_server_timestamp_delta = Some(match self.average_server_timestamp_delta {
            Some(average_server_timestamp_delta) => {
                (average_server_timestamp_delta * 4.0 + timestamp_delta) / 5.0
            }
            None => timestamp_delta,
        });
    }
}

lazy_static! {
    pub static ref CONNECTED_CLIENTS: ConnectedClients = ConnectedClients::default();
    pub static ref CONNECTED_ACCOUNTS: ConnectedAccounts = ConnectedAccounts::default();
}

pub struct ConnectedClients {
    data: Arc<RwLock<ConnectedClientData>>,
}

pub struct ConnectedClientData {
    clients: HashMap<Uuid, Arc<RwLock<ConnectedClient>>>,
    installations_by_account: HashMap<i64, HashSet<Uuid>>,
    account_by_installation: HashMap<Uuid, i64>,
}

impl Default for ConnectedClients {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(ConnectedClientData {
                clients: HashMap::new(),
                installations_by_account: HashMap::new(),
                account_by_installation: HashMap::new(),
            })),
        }
    }
}

impl ConnectedClients {
    pub async fn connect(&self, installation_id: Uuid, client: &Arc<RwLock<ConnectedClient>>) {
        let mut data = self.data.write().await;
        data.clients.insert(installation_id, client.clone());
        {
            let mut client = client.write().await;
            client.installation_id = Some(installation_id);
        }
    }

    pub async fn associate_account(
        &self,
        installation_id: Uuid,
        account_id: i64,
    ) -> Result<Arc<RwLock<ConnectedAccount>>, anyhow::Error> {
        let mut data = self.data.write().await;
        let account = CONNECTED_ACCOUNTS.connect(installation_id).await?;
        if let Some(client) = data.clients.get_mut(&installation_id) {
            let mut client = client.write().await;
            client.account = Some(account.clone());
        }

        data.account_by_installation
            .insert(installation_id, account_id);
        let installations = data
            .installations_by_account
            .entry(account_id)
            .or_insert_with(HashSet::new);
        installations.insert(installation_id);
        Ok(account)
    }

    pub async fn disconnect(&self, installation_id: Uuid) {
        info!("Disconnecting installation {}", installation_id);
        let mut data = self.data.write().await;

        data.clients.remove(&installation_id);
        let account_id = match data.account_by_installation.get(&installation_id) {
            Some(account_id) => *account_id,
            None => return,
        };

        let remove_account =
            if let Some(installations) = data.installations_by_account.get_mut(&account_id) {
                installations.remove(&installation_id);
                installations.is_empty()
            } else {
                false
            };
        if remove_account {
            data.installations_by_account.remove(&account_id);
            CONNECTED_ACCOUNTS.fully_disconnected(account_id).await;
        }
    }

    pub async fn send_to_installation_id(&self, installation_id: Uuid, message: ServerResponse) {
        let data = self.data.read().await;
        if let Some(client) = data.clients.get(&installation_id) {
            let client = client.read().await;
            client
                .sender
                .send(message.into_ws_response(-1))
                .unwrap_or_default();
        }
    }

    pub async fn send_to_account_id(&self, account_id: i64, message: ServerResponse) {
        let data = self.data.read().await;
        if let Some(installation_ids) = data.installations_by_account.get(&account_id) {
            for installation_id in installation_ids.iter() {
                if let Some(client) = data.clients.get(&installation_id) {
                    let client = client.read().await;
                    client
                        .sender
                        .send(message.clone().into_ws_response(-1))
                        .unwrap_or_default();
                }
            }
        }
    }

    // pub async fn world_updated(&self, update_timestamp: f64) -> Result<(), anyhow::Error> {
    //     todo!()s
    //     // let world_update = ServerResponse::WorldUpdate {
    //     //     timestamp: update_timestamp,
    //     //     profiles: sqlx::query_as!(
    //     //         UserProfile,
    //     //         "SELECT id, username, map, x_offset, last_update_timestamp, horizontal_input from account_list_current($1)",
    //     //         update_timestamp - 5.0
    //     //     )
    //     //     .fetch_all(&pg())
    //     //     .await?,
    //     // };

    //     // let clients = self.clients.read().await;
    //     // for (_, client) in clients.iter() {
    //     //     let client = client.read().await;
    //     //     client.sender.send(world_update.clone()).unwrap_or_default();
    //     // }
    //     // Ok(())
    // }

    pub async fn ping(&self) {
        let data = self.data.read().await;
        let timestamp = current_timestamp();
        for client in data.clients.values() {
            let client = client.read().await;
            client
                .sender
                .send(
                    ServerResponse::Ping {
                        timestamp,
                        average_roundtrip: client
                            .network_timing
                            .average_roundtrip
                            .unwrap_or_default(),
                        average_server_timestamp_delta: client
                            .network_timing
                            .average_server_timestamp_delta
                            .unwrap_or_default(),
                    }
                    .into_ws_response(-1),
                )
                .unwrap_or_default();
        }
    }
}

pub struct ConnectedClient {
    installation_id: Option<Uuid>,
    sender: UnboundedSender<WsBatchResponse>,
    account: Option<Arc<RwLock<ConnectedAccount>>>,
    network_timing: NetworkTiming,
}

pub struct ConnectedAccounts {
    accounts_by_id: Arc<RwLock<HashMap<i64, Arc<RwLock<ConnectedAccount>>>>>,
}

impl Default for ConnectedAccounts {
    fn default() -> Self {
        Self {
            accounts_by_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl ConnectedAccounts {
    pub async fn connect(
        &self,
        installation_id: Uuid,
    ) -> Result<Arc<RwLock<ConnectedAccount>>, anyhow::Error> {
        let mut accounts_by_id = self.accounts_by_id.write().await;

        let profile = database::get_profile_by_installation_id(&pg(), installation_id).await?;

        // TODO it'd be nice to not do this unless we need to do it, but I don't think it's possible without using a block_on.
        let permissions = database::load_permissions_for(&pg(), profile.id).await?;

        Ok(accounts_by_id
            .entry(profile.id)
            .or_insert_with(|| {
                Arc::new(RwLock::new(ConnectedAccount {
                    profile,
                    inputs: None,
                    permissions,
                }))
            })
            .clone())
    }

    pub async fn fully_disconnected(&self, account_id: i64) {
        info!("Disconnecting account {}", account_id);
        let mut accounts_by_id = self.accounts_by_id.write().await;
        accounts_by_id.remove(&account_id);
    }

    pub async fn role_updated(&self, role_id: i64) -> Result<(), anyhow::Error> {
        info!("Updating role: {}", role_id);
        let accounts_to_refresh = {
            let accounts_by_id = self.accounts_by_id.read().await;
            let mut accounts_to_refresh = Vec::new();
            for account_handle in accounts_by_id.values() {
                let account = account_handle.read().await;
                if account.permissions.role_ids.contains(&role_id) {
                    accounts_to_refresh.push(account.profile.id);
                }
            }
            accounts_to_refresh
        };

        for account_id in accounts_to_refresh {
            tokio::spawn(async move {
                info!("Updating account: {}", account_id);
                Self::notify_account_updated(account_id)
                    .await
                    .unwrap_or_default()
            });
        }

        Ok(())
    }

    async fn notify_account_updated(account_id: i64) -> Result<(), anyhow::Error> {
        let pg = pg();
        let profile = crate::database::get_profile_by_account_id(&pg, account_id).await?;
        let permissions = crate::database::load_permissions_for(&pg, account_id).await?;
        info!("Got new permissions: {:?}", permissions);
        CONNECTED_ACCOUNTS
            .update_account_permissions(account_id, permissions.clone())
            .await;
        CONNECTED_CLIENTS
            .send_to_account_id(
                account_id,
                ServerResponse::Authenticated {
                    permissions,
                    profile,
                },
            )
            .await;

        Ok(())
    }

    async fn update_account_permissions(&self, account_id: i64, permissions: PermissionSet) {
        let accounts_by_id = self.accounts_by_id.read().await;
        if let Some(account_handle) = accounts_by_id.get(&account_id) {
            let mut account = account_handle.write().await;
            account.permissions = permissions;
        }
    }

    // pub async fn all(&self) -> Vec<Arc<RwLock<ConnectedAccount>>> {
    //     let accounts_by_id = self.accounts_by_id.read().await;
    //     accounts_by_id.values().map(|v| v.clone()).collect()
    // }
}

#[async_trait]
pub trait ConnectedAccountHandle {
    async fn permission_allowed(&self, claim: &Claim) -> Result<(), anyhow::Error>;
}

fn permission_denied(claim: &Claim) -> Result<(), anyhow::Error> {
    anyhow::bail!("permission denied for accessing {:?}", claim)
}

#[async_trait]
impl ConnectedAccountHandle for Arc<RwLock<ConnectedClient>> {
    async fn permission_allowed(&self, claim: &Claim) -> Result<(), anyhow::Error> {
        let client = self.read().await;
        if let Some(account) = &client.account {
            return account.permission_allowed(claim).await;
        }

        permission_denied(claim)
    }
}

#[async_trait]
impl ConnectedAccountHandle for Arc<RwLock<ConnectedAccount>> {
    async fn permission_allowed(&self, claim: &Claim) -> Result<(), anyhow::Error> {
        let account = self.read().await;
        if account.permissions.allowed(&claim) {
            Ok(())
        } else {
            permission_denied(claim)
        }
    }
}

pub struct ConnectedAccount {
    pub profile: UserProfile,
    pub permissions: PermissionSet,
    pub inputs: Option<Inputs>,
}

impl ConnectedAccount {
    // pub async fn set_x_offset(
    //     &mut self,
    //     x_offset: f32,
    //     timestamp: f64,
    // ) -> Result<(), anyhow::Error> {
    //     // TODO THE AVERAGE VELOCITY CANNOT EXCEED THE MAXIMUM IN CODE
    //     // self.profile.x_offset = x_offset;
    //     // self.profile.horizontal_input = self
    //     //     .inputs
    //     //     .as_ref()
    //     //     .map(|inputs| inputs.horizontal_movement)
    //     //     .unwrap_or_default();
    //     // let _ = sqlx::query!(
    //     //     "SELECT * FROM account_update_inputs($1, $2, $3, $4)",
    //     //     self.profile.id,
    //     //     self.profile.x_offset,
    //     //     timestamp,
    //     //     self.profile.horizontal_input
    //     // )
    //     // .fetch_one(&pg())
    //     // .await?;

    //     // Ok(())
    //     todo!()
    // }
}

pub async fn initialize() {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            CONNECTED_CLIENTS.ping().await;
        }
    });
}

pub async fn main(websocket: WebSocket) {
    let (mut tx, mut rx) = websocket.split();
    let (sender, mut transmission_receiver) = unbounded_channel();

    tokio::spawn(async move {
        while let Some(response) = transmission_receiver.recv().await {
            tx.send(Message::binary(bincode::serialize(&response).unwrap()))
                .await
                .unwrap_or_default()
        }
    });

    let client = Arc::new(RwLock::new(ConnectedClient {
        account: None,
        installation_id: None,
        network_timing: NetworkTiming::default(),
        sender: sender.clone(),
    }));
    while let Some(result) = rx.next().await {
        match result {
            Ok(message) => match bincode::deserialize::<WsRequest>(message.as_bytes()) {
                Ok(request) => {
                    let request_id = request.id;
                    if let Err(err) =
                        handle_websocket_request(&client, request, sender.clone()).await
                    {
                        sender
                            .send(
                                ServerResponse::Error {
                                    message: Some(err.to_string()),
                                }
                                .into_ws_response(request_id),
                            )
                            .unwrap_or_default();
                    }
                }
                Err(err) => {
                    error!("Bincode error: {}", err);
                    break;
                }
            },
            Err(err) => {
                error!("Error on websocket: {}", err);
                break;
            }
        }
    }

    let installation_id = {
        let client_data = client.read().await;
        client_data.installation_id
    };

    if let Some(installation_id) = installation_id {
        CONNECTED_CLIENTS.disconnect(installation_id).await;
    }
}

async fn handle_websocket_request(
    client_handle: &Arc<RwLock<ConnectedClient>>,
    request: WsRequest,
    responder: UnboundedSender<WsBatchResponse>,
) -> Result<(), anyhow::Error> {
    match request.request {
        // ServerRequest::Update {
        //     new_inputs,
        //     x_offset,
        //     timestamp,
        // } => {
        //     let client = client_handle.read().await;
        //     let corrected_server_timestamp = timestamp
        //         + client
        //             .network_timing
        //             .average_server_timestamp_delta
        //             .unwrap_or_default();
        //     let now = current_timestamp();

        //     if let Some(account) = &client.account {
        //         let mut account = account.write().await;
        //         account.inputs = new_inputs;
        //         let latency_corrected_x_offset =
        //             x_offset + WALK_SPEED * (now - corrected_server_timestamp) as f32;
        //         account
        //             .set_x_offset(latency_corrected_x_offset, now)
        //             .await?;
        //     }
        // }
        ServerRequest::Authenticate {
            installation_id,
            version,
        } => {
            if version != shared::PROTOCOL_VERSION {
                responder
                    .send(
                        ServerResponse::Error {
                            message: Some("An update is available".to_owned()),
                        }
                        .into_ws_response(request.id),
                    )
                    .unwrap_or_default();
                return Ok(());
            }

            let installation_id = match installation_id {
                Some(installation_id) => installation_id,
                None => Uuid::new_v4(),
            };

            info!("Looking up installation {:?}", installation_id);
            let installation = database::lookup_installation(&pg(), installation_id).await?;

            info!("Recording connection");
            CONNECTED_CLIENTS
                .connect(installation.id, &client_handle)
                .await;

            info!("Looking up account");
            let logged_in = if let Some(account_id) = installation.account_id {
                if let Ok(profile) =
                    database::get_profile_by_installation_id(&pg(), installation.id).await
                {
                    let account = CONNECTED_CLIENTS
                        .associate_account(installation.id, account_id)
                        .await?;
                    let account = account.read().await;
                    if !account
                        .permissions
                        .allowed(&Claim::new("ncog", None, None, "connect"))
                    {
                        responder
                            .send(
                                ServerResponse::Error {
                                    message: Some(
                                        "You have been banned from connecting.".to_owned(),
                                    ),
                                }
                                .into_ws_response(request.id),
                            )
                            .unwrap_or_default();
                        return Ok(());
                    }

                    responder
                        .send(
                            ServerResponse::Authenticated {
                                profile,
                                permissions: account.permissions.clone(),
                            }
                            .into_ws_response(request.id),
                        )
                        .unwrap_or_default();
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if !logged_in {
                // The user is not logged in, send the adopt installation ID as an indicator that we're not authenticated
                responder
                    .send(
                        ServerResponse::AdoptInstallationId {
                            installation_id: installation.id,
                        }
                        .into_ws_response(request.id),
                    )
                    .unwrap_or_default();
            }
        }
        ServerRequest::AuthenticationUrl(provider) => match provider {
            OAuthProvider::Twitch => {
                let client = client_handle.read().await;
                if let Some(installation_id) = client.installation_id {
                    responder
                        .send(
                            ServerResponse::AuthenticateAtUrl {
                                url: twitch::authorization_url(installation_id),
                            }
                            .into_ws_response(request.id),
                        )
                        .unwrap_or_default();
                }
            }
        },
        ServerRequest::Pong {
            original_timestamp,
            timestamp,
        } => {
            let mut client = client_handle.write().await;
            client.network_timing.update(original_timestamp, timestamp);
        }
        ServerRequest::IAM(iam_request) => {
            iam::handle_request(client_handle, iam_request, responder, request.id).await?;
        }
    }

    Ok(())
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
    // pub display_name: Option<String>,
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

    let pg = pg();
    {
        let mut tx = pg.begin().await?;

        // Create an account if it doesn't exist yet for this installation
        let account_id = if let Some(account_id) = sqlx::query!(
            "SELECT account_id FROM installations WHERE id = $1",
            installation_id
        )
        .fetch_one(&mut tx)
        .await?
        .account_id
        {
            account_id
        } else {
            let account_id = if let Ok(row) = sqlx::query!(
                "SELECT account_id FROM twitch_profiles WHERE id = $1",
                user.id
            )
            .fetch_one(&mut tx)
            .await
            {
                row.account_id
            } else {
                sqlx::query!("INSERT INTO accounts DEFAULT VALUES RETURNING id")
                    .fetch_one(&mut tx)
                    .await?
                    .id
            };
            sqlx::query!(
                "UPDATE installations SET account_id = $1, nonce = NULL WHERE id = $2",
                account_id,
                installation_id
            )
            .execute(&mut tx)
            .await?;
            account_id
        };

        // Create an twitch profile
        sqlx::query!("INSERT INTO twitch_profiles (id, account_id, username) VALUES ($1, $2, $3) ON CONFLICT (id) DO UPDATE SET account_id = $2, username = $3 ",
            user.id,
            account_id,
            user.login,
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
