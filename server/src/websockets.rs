use super::{database, env};
use async_std::sync::RwLock;
use crossbeam::channel::{unbounded, Sender};
use futures::{executor::block_on, SinkExt, StreamExt};
use lazy_static::lazy_static;
use migrations::pg;
use shared::{
    current_timestamp,
    websockets::{WsBatchResponse, WsRequest},
    Inputs, OAuthProvider, ServerRequest, ServerResponse, UserProfile,
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
        println!("connect:locking");
        let mut data = self.data.write().await;
        data.clients.insert(installation_id, client.clone());
        {
            let mut client = client.write().await;
            client.installation_id = Some(installation_id);
        }
        println!("connect:exiting");
    }

    pub async fn associate_account(
        &self,
        installation_id: Uuid,
        account_id: i64,
    ) -> Result<(), anyhow::Error> {
        println!("associat_account:locking");
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
        println!("associate_account:returning");
        Ok(())
    }

    pub async fn disconnect(&self, installation_id: Uuid) {
        println!("disconnect:locking");
        let mut data = self.data.write().await;
        println!("disconnect:locked6");

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
        println!("send_to_installation_id:locking");
        let data = self.data.read().await;
        if let Some(client) = data.clients.get(&installation_id) {
            let client = client.read().await;
            client
                .sender
                .send(message.into_ws_response(-1))
                .unwrap_or_default();
        }
        println!("send_to_installation_id:exiting");
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
        println!("ping:locking");
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
        println!("ping:exiting");
    }
}

pub struct ConnectedClient {
    installation_id: Option<Uuid>,
    sender: Sender<WsBatchResponse>,
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
        println!("Fully disconnected.");
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
    println!("Websocket main");
    let (mut tx, mut rx) = websocket.split();
    let (sender, transmission_receiver) = unbounded();

    tokio::spawn(async move {
        while let Ok(response) = transmission_receiver.recv() {
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
        println!("Received message from websocket");
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
                    println!("Bincode error: {}", err);
                    return;
                }
            },
            Err(err) => {
                println!("Error on websocket: {}", err);
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
    responder: Sender<WsBatchResponse>,
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

            println!("Looking up installation {:?}", installation_id);
            let installation = database::lookup_installation(&pg(), installation_id).await?;

            println!("Recording connection");
            CONNECTED_CLIENTS
                .connect(installation.id, &client_handle)
                .await;

            println!("Looking up account");
            let logged_in = if let Some(account_id) = installation.account_id {
                if let Ok(profile) = database::get_profile(&pg(), installation.id).await {
                    if !database::check_permission(&pg(), account_id, "ncog", None, None, "connect")
                        .await?
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

                    CONNECTED_CLIENTS
                        .associate_account(installation.id, account_id)
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
                println!("Sending auth url");
                if let Some(installation_id) = client.installation_id {
                    println!("Sending authentication url");
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

#[cfg(debug_assertions)]
static REDIRECT_URI: &'static str = "http://localhost:7878/api/auth/itchio_callback";
#[cfg(not(debug_assertions))]
static REDIRECT_URI: &'static str = "https://ncog.live/api/auth/itchio_callback";

fn itchio_authorization_url(installation_id: Uuid) -> String {
    Url::parse_with_params(
        "https://itch.io/user/oauth",
        &[
            ("client_id", env("ITCHIO_CLIENT_ID")),
            ("scope", "profile:me".to_owned()),
            ("response_type", "token".to_owned()),
            ("redirect_uri", REDIRECT_URI.to_owned()),
            ("state", installation_id.to_string()),
        ],
    )
    .unwrap()
    .to_string()
}
