use super::env;
use async_std::sync::RwLock;
use crossbeam::channel::{unbounded, Sender};
use futures::{executor::block_on, SinkExt, StreamExt};
use lazy_static::lazy_static;
use migrations::{pg, sqlx};
use shared::{
    current_timestamp, Inputs, Installation, ServerRequest, ServerResponse, UserProfile, WALK_SPEED,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
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
    clients: Arc<RwLock<HashMap<Uuid, Arc<RwLock<ConnectedClient>>>>>,
    installations_by_account: Arc<RwLock<HashMap<i64, HashSet<Uuid>>>>,
    account_by_installation: Arc<RwLock<HashMap<Uuid, i64>>>,
}

impl Default for ConnectedClients {
    fn default() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            installations_by_account: Arc::new(RwLock::new(HashMap::new())),
            account_by_installation: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl ConnectedClients {
    pub async fn connect(&self, installation_id: Uuid, client: &Arc<RwLock<ConnectedClient>>) {
        {
            let mut client = client.write().await;
            client.installation_id = Some(installation_id);
        }
        let mut clients = self.clients.write().await;
        clients.insert(installation_id, client.clone());
    }

    pub async fn associate_account(
        &self,
        installation_id: Uuid,
        account_id: i64,
    ) -> Result<(), anyhow::Error> {
        let mut installations_by_account = self.installations_by_account.write().await;
        let mut account_by_installation = self.account_by_installation.write().await;
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(&installation_id) {
            let mut client = client.write().await;
            client.account = Some(CONNECTED_ACCOUNTS.connect(installation_id).await?);
        }

        account_by_installation.insert(installation_id, account_id);
        let installations = installations_by_account
            .entry(account_id)
            .or_insert_with(|| HashSet::new());
        installations.insert(installation_id);
        Ok(())
    }

    pub async fn disconnect(&self, installation_id: Uuid) {
        let mut installations_by_account = self.installations_by_account.write().await;
        let account_by_installation = self.account_by_installation.read().await;
        let mut clients = self.clients.write().await;

        clients.remove(&installation_id);
        if let Some(account_id) = account_by_installation.get(&installation_id) {
            let remove_account =
                if let Some(installations) = installations_by_account.get_mut(account_id) {
                    installations.remove(&installation_id);
                    installations.len() == 0
                } else {
                    false
                };
            if remove_account {
                installations_by_account.remove(account_id);
                CONNECTED_ACCOUNTS.fully_disconnected(*account_id).await;
            }
        }
    }

    pub async fn send_to_installation_id(&self, installation_id: Uuid, message: ServerResponse) {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(&installation_id) {
            let client = client.read().await;
            client.sender.send(message).unwrap_or_default();
        }
    }

    pub async fn world_updated(&self, update_timestamp: f64) -> Result<(), anyhow::Error> {
        let world_update = ServerResponse::WorldUpdate {
            timestamp: update_timestamp,
            profiles: sqlx::query_as!(
                UserProfile,
                "SELECT id, username, map, x_offset, last_update_timestamp, horizontal_input from account_list_current($1)",
                update_timestamp - 5.0
            )
            .fetch_all(&pg())
            .await?,
        };

        let clients = self.clients.read().await;
        for (_, client) in clients.iter() {
            let client = client.read().await;
            client.sender.send(world_update.clone()).unwrap_or_default();
        }
        Ok(())
    }

    pub async fn ping(&self) {
        let clients = self.clients.read().await;
        let timestamp = current_timestamp();
        for client in clients.values() {
            let client = client.read().await;
            client
                .sender
                .send(ServerResponse::Ping {
                    timestamp,
                    average_roundtrip: client.network_timing.average_roundtrip.unwrap_or_default(),
                    average_server_timestamp_delta: client
                        .network_timing
                        .average_server_timestamp_delta
                        .unwrap_or_default(),
                })
                .unwrap_or_default();
        }
    }
}

pub struct ConnectedClient {
    installation_id: Option<Uuid>,
    sender: Sender<ServerResponse>,
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

        let profile = sqlx::query_as!(
            UserProfile,
            "SELECT id, username, map, x_offset, last_update_timestamp, horizontal_input FROM installation_profile($1)",
            installation_id,
        )
        .fetch_one(&pg())
        .await?;

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

    pub async fn all(&self) -> Vec<Arc<RwLock<ConnectedAccount>>> {
        let accounts_by_id = self.accounts_by_id.read().await;
        accounts_by_id.values().map(|v| v.clone()).collect()
    }
}

pub struct ConnectedAccount {
    pub profile: UserProfile,
    pub inputs: Option<Inputs>,
}

impl ConnectedAccount {
    pub async fn set_x_offset(
        &mut self,
        x_offset: f32,
        timestamp: f64,
    ) -> Result<(), anyhow::Error> {
        // TODO THE AVERAGE VELOCITY CANNOT EXCEED THE MAXIMUM IN CODE
        self.profile.x_offset = x_offset;
        self.profile.horizontal_input = self
            .inputs
            .as_ref()
            .map(|inputs| inputs.horizontal_movement)
            .unwrap_or_default();
        let _ = sqlx::query!(
            "SELECT * FROM account_update_inputs($1, $2, $3, $4)",
            self.profile.id,
            self.profile.x_offset,
            timestamp,
            self.profile.horizontal_input
        )
        .fetch_one(&pg())
        .await?;

        Ok(())
    }
}

pub async fn main(websocket: WebSocket) {
    let (mut tx, mut rx) = websocket.split();
    let (sender, transmission_receiver) = unbounded();

    tokio::spawn(async move {
        while let Ok(response) = transmission_receiver.recv() {
            tx.send(Message::binary(bincode::serialize(&response).unwrap()))
                .await
                .unwrap_or_default()
        }
    });

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            CONNECTED_CLIENTS.ping().await;
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
            Ok(message) => match bincode::deserialize::<ServerRequest>(message.as_bytes()) {
                Ok(request) => {
                    if let Err(err) =
                        handle_websocket_request(&client, request, sender.clone()).await
                    {
                        sender
                            .send(ServerResponse::Error {
                                message: Some(err.to_string()),
                            })
                            .unwrap_or_default();
                    }
                }
                Err(err) => println!("Bincode error: {}", err),
            },
            Err(err) => {
                println!("Error on websocket: {}", err);
                return;
            }
        }
    }
}

async fn handle_websocket_request(
    client_handle: &Arc<RwLock<ConnectedClient>>,
    request: ServerRequest,
    responder: Sender<ServerResponse>,
) -> Result<(), anyhow::Error> {
    match request {
        ServerRequest::Update {
            new_inputs,
            x_offset,
            timestamp,
        } => {
            let client = client_handle.read().await;
            let corrected_server_timestamp = timestamp
                + client
                    .network_timing
                    .average_server_timestamp_delta
                    .unwrap_or_default();
            let now = current_timestamp();

            if let Some(account) = &client.account {
                let mut account = account.write().await;
                account.inputs = new_inputs;
                let latency_corrected_x_offset =
                    x_offset + WALK_SPEED * (now - corrected_server_timestamp) as f32;
                account
                    .set_x_offset(latency_corrected_x_offset, now)
                    .await?;
            }
        }
        ServerRequest::Authenticate {
            installation_id,
            version,
        } => {
            if &version != shared::PROTOCOL_VERSION {
                responder
                    .send(ServerResponse::Error {
                        message: Some("An update is available".to_owned()),
                    })
                    .unwrap_or_default();
                return Ok(());
            }

            let installation_id = Some(match installation_id {
                Some(installation_id) => installation_id,
                None => {
                    let installation_id = Uuid::new_v4();
                    responder
                        .send(ServerResponse::AdoptInstallationId {
                            installation_id: installation_id,
                        })
                        .unwrap_or_default();
                    installation_id
                }
            });

            let pool = pg();
            let installation = sqlx::query_as!(
                Installation,
                "SELECT * FROM installation_lookup($1)",
                installation_id
            )
            .fetch_one(&pool)
            .await?;

            CONNECTED_CLIENTS
                .connect(installation.id, &client_handle)
                .await;

            if let Some(account_id) = installation.account_id {
                let profile = sqlx::query_as!(
                        UserProfile,
                        "SELECT id, username, map, x_offset, last_update_timestamp, horizontal_input FROM installation_profile($1)",
                        installation.id,
                    )
                    .fetch_one(&pool)
                    .await?;

                CONNECTED_CLIENTS
                    .associate_account(installation.id, account_id)
                    .await?;
                responder
                    .send(ServerResponse::Authenticated { profile })
                    .unwrap_or_default();
            }
        }
        ServerRequest::AuthenticationUrl => {
            let client = client_handle.read().await;
            if let Some(installation_id) = client.installation_id {
                responder
                    .send(ServerResponse::AuthenticateAtUrl {
                        url: itchio_authorization_url(installation_id),
                    })
                    .unwrap_or_default();
            }
        }
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

impl Drop for ConnectedClient {
    fn drop(&mut self) {
        if let Some(installation_id) = self.installation_id {
            block_on(CONNECTED_CLIENTS.disconnect(installation_id));
        }
    }
}

#[cfg(debug_assertions)]
static REDIRECT_URI: &'static str = "http://localhost:7878/auth/itchio_callback";
#[cfg(not(debug_assertions))]
static REDIRECT_URI: &'static str = "https://cantina.khonsu.gg/auth/itchio_callback";

fn itchio_authorization_url(installation_id: Uuid) -> String {
    Url::parse_with_params(
        "https://itch.io/user/oauth",
        &[
            ("client_id", env("OAUTH_CLIENT_ID")),
            ("scope", "profile:me".to_owned()),
            ("response_type", "token".to_owned()),
            ("redirect_uri", REDIRECT_URI.to_owned()),
            ("state", installation_id.to_string()),
        ],
    )
    .unwrap()
    .to_string()
}
