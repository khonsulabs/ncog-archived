use super::{database, env, SERVER_URL};
use crate::permissions::{Claim, PermissionSet};
use async_std::sync::RwLock;
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use migrations::{pg, sqlx};
use serde_derive::{Deserialize, Serialize};
use shared::{
    current_timestamp,
    websockets::{WsBatchResponse, WsRequest},
    Inputs, OAuthProvider, ServerRequest, ServerResponse, UserProfile,
};
use sqlx::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use url::Url;
use uuid::Uuid;
use warp::filters::ws::{Message, WebSocket};

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
    pub static ref CONNECTED_CLIENTS: ConnectedClients = { ConnectedClients::default() };
    pub static ref CONNECTED_ACCOUNTS: ConnectedAccounts = { ConnectedAccounts::default() };
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
        permissions: PermissionSet,
    ) -> Result<(), anyhow::Error> {
        let mut data = self.data.write().await;
        if let Some(client) = data.clients.get_mut(&installation_id) {
            let mut client = client.write().await;
            client.account = Some(CONNECTED_ACCOUNTS.connect(installation_id).await?);
        }

        data.account_by_installation
            .insert(installation_id, account_id);
        let installations = data
            .installations_by_account
            .entry(account_id)
            .or_insert_with(|| HashSet::new());
        installations.insert(installation_id);
        Ok(())
    }

    pub async fn disconnect(&self, installation_id: Uuid) {
        let mut data = self.data.write().await;

        data.clients.remove(&installation_id);
        let account_id = match data.account_by_installation.get(&installation_id) {
            Some(account_id) => *account_id,
            None => return,
        };

        let remove_account =
            if let Some(installations) = data.installations_by_account.get_mut(&account_id) {
                installations.remove(&installation_id);
                installations.len() == 0
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

        let profile = database::get_profile(&pg(), installation_id).await?;

        Ok(accounts_by_id
            .entry(profile.id)
            .or_insert_with(|| {
                Arc::new(RwLock::new(ConnectedAccount {
                    profile,
                    inputs: None,
                }))
            })
            .clone())
    }

    pub async fn fully_disconnected(&self, account_id: i64) {
        let mut accounts_by_id = self.accounts_by_id.write().await;
        accounts_by_id.remove(&account_id);
    }

    // pub async fn all(&self) -> Vec<Arc<RwLock<ConnectedAccount>>> {
    //     let accounts_by_id = self.accounts_by_id.read().await;
    //     accounts_by_id.values().map(|v| v.clone()).collect()
    // }
}

pub struct ConnectedAccount {
    pub profile: UserProfile,
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
                    return;
                }
            },
            Err(err) => {
                error!("Error on websocket: {}", err);
                return;
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
        ServerRequest::ReceiveItchIOAuth {
            access_token,
            state,
        } => {
            let installation_id = match Uuid::parse_str(&state) {
                Ok(uuid) => uuid,
                Err(_) => {
                    error!("Invalid UUID in state");
                    todo!("Report error back to client");
                }
            };
            if let Err(err) = login_itchio(installation_id, &access_token).await {
                error!(
                    "Error logging into itch.io; {}, {}: {}",
                    installation_id, access_token, err
                );
                responder
                    .send(ServerResponse::Error {
                        message: Some(
                            "An error occurred while talking to itch.io. Please try again later."
                                .to_owned(),
                        ),
                    }.into_ws_response(request.id))
                    .unwrap_or_default();
            }
        }
        ServerRequest::Authenticate {
            installation_id,
            version,
        } => {
            if &version != shared::PROTOCOL_VERSION {
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

            trace!("Looking up installation {:?}", installation_id);
            let installation = database::lookup_installation(&pg(), installation_id).await?;

            trace!("Recording connection");
            CONNECTED_CLIENTS
                .connect(installation.id, &client_handle)
                .await;

            trace!("Looking up account");
            let logged_in = if let Some(account_id) = installation.account_id {
                if let Ok(profile) = database::get_profile(&pg(), installation.id).await {
                    let account_permissions =
                        database::load_permissions_for(&pg(), account_id).await?;
                    if !account_permissions.allowed(&Claim::new("ncog", None, None, "connect")) {
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

                    CONNECTED_CLIENTS
                        .associate_account(installation.id, account_id, account_permissions)
                        .await?;
                    responder
                        .send(
                            ServerResponse::Authenticated { profile }.into_ws_response(request.id),
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
            OAuthProvider::ItchIO => {
                let client = client_handle.read().await;
                if let Some(installation_id) = client.installation_id {
                    responder
                        .send(
                            ServerResponse::AuthenticateAtUrl {
                                url: itchio_authorization_url(installation_id),
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
    }

    Ok(())
}

fn itchio_authorization_url(installation_id: Uuid) -> String {
    Url::parse_with_params(
        "https://itch.io/user/oauth",
        &[
            ("client_id", env("ITCHIO_CLIENT_ID")),
            ("scope", "profile:me".to_owned()),
            ("response_type", "token".to_owned()),
            (
                "redirect_uri",
                format!("{}/auth/callback/itchio", SERVER_URL),
            ),
            ("state", installation_id.to_string()),
        ],
    )
    .unwrap()
    .to_string()
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
struct ItchioProfileResponse {
    pub user: ItchioProfile,
}

async fn login_itchio(installation_id: Uuid, access_token: &String) -> Result<(), anyhow::Error> {
    // Call itch.io API to get the user information
    let client = reqwest::Client::new();
    let response: ItchioProfileResponse = client
        .get("https://itch.io/api/1/key/me")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await?
        .json()
        .await?;

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
            let account_id = sqlx::query!("INSERT INTO accounts DEFAULT VALUES RETURNING id")
                .fetch_one(&mut tx)
                .await?
                .id;
            sqlx::query!(
                "UPDATE installations SET account_id = $1 WHERE id = $2",
                account_id,
                installation_id
            )
            .execute(&mut tx)
            .await?;
            account_id
        };

        // Create an itchio profile
        sqlx::query!("INSERT INTO itchio_profiles (id, account_id, username, url) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET account_id = $2, username = $3, url = $4 ",
            response.user.id,
            account_id,
            response.user.username,
            response.user.url
        ).execute(&mut tx).await?;

        // Create an oauth_token
        sqlx::query!("INSERT INTO oauth_tokens (account_id, service, access_token) VALUES ($1, $2, $3) ON CONFLICT (account_id, service) DO UPDATE SET access_token = $3",
            account_id,
            "itchio",
            access_token
        ).execute(&mut tx).await?;

        tx.commit().await?;
    }

    let mut connection = pg.acquire().await?;
    connection
        .execute(&*format!(
            "NOTIFY installation_login, '{}'",
            installation_id
        ))
        .await?;
    Ok(())
}
