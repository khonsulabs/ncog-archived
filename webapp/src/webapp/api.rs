use khonsuweb::wasm_utc_now;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use shared::{
    websockets::{WsBatchResponse, WsRequest, WsResponse},
    ServerRequest, ServerResponse, UserProfile,
};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;
use yew::prelude::*;
use yew::worker::*;
use yew::{
    format::Json,
    services::{
        storage::{Area, StorageService},
        timeout::{TimeoutService, TimeoutTask},
        websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
    },
};
use yew_router::{
    agent::{RouteAgentBridge, RouteRequest},
    route::Route,
};

pub enum Message {
    Initialize,
    Reset,
    Message(WsBatchResponse),
    Connected,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AgentResponse {
    Connected,
    Disconnected,
    Response(WsResponse),
    StorageStatus(bool),
}

pub struct ApiAgent {
    link: AgentLink<Self>,
    web_socket_task: Option<WebSocketTask>,
    ws_request_id: i64,
    reconnect_timer: Option<TimeoutTask>,
    reconnect_sleep_ms: u32,
    callbacks: HashMap<i64, HandlerId>,
    broadcasts: HashSet<HandlerId>,
    ready_for_messages: bool,
    storage: StorageService,
    auth_state: AuthState,
    storage_enabled: bool,
}

pub type ApiBridge = Box<dyn Bridge<ApiAgent>>;

const DEFAULT_RECONNECT_TIMEOUT: u32 = 125;
#[derive(Serialize, Deserialize, Debug)]
pub enum AgentMessage {
    Request(ServerRequest),
    Initialize,
    Reset,
    RegisterBroadcastHandler,
    UnregisterBroadcastHandler,
    LogOut,
    QueryStorageStatus,
    EnableStorage,
    DisableStorage,
}

impl Agent for ApiAgent {
    type Reach = Context<Self>; // Spawn only one instance on the main thread (all components can share this agent)
    type Message = Message;
    type Input = AgentMessage;
    type Output = AgentResponse;

    // Create an instance with a link to the agent.
    fn create(link: AgentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).expect("Error accessing storage service");
        let Json(login_state) = storage.restore("login_state");
        let encrypted_login_info: EncryptedLoginInformation = login_state.unwrap_or_default();
        let auth_state = encrypted_login_info.auth_state();
        let storage_enabled = auth_state != AuthState::Unauthenticated;
        Self {
            link,
            web_socket_task: None,
            ws_request_id: 0,
            reconnect_sleep_ms: DEFAULT_RECONNECT_TIMEOUT,
            reconnect_timer: None,
            callbacks: HashMap::new(),
            broadcasts: HashSet::new(),
            ready_for_messages: false,
            storage,
            auth_state,
            storage_enabled,
        }
    }

    // Handle inner messages (from callbacks)
    fn update(&mut self, msg: Self::Message) {
        match msg {
            Message::Initialize => {
                self.reconnect_timer = None;
                self.initialize_websockets();
            }
            Message::Connected => {
                self.reconnect_sleep_ms = DEFAULT_RECONNECT_TIMEOUT;
                self.ready_for_messages = true;
                self.ws_send(
                    ServerRequest::Authenticate {
                        version: shared::PROTOCOL_VERSION.to_owned(),
                        installation_id: self.installation_id(),
                    },
                    None,
                );
                self.broadcast(AgentResponse::Connected);
            }
            Message::Message(ws_response) => {
                let request_id = ws_response.request_id;
                for individual_result in ws_response.results.iter() {
                    self.handle_ws_message(&individual_result);
                    let individual_result = AgentResponse::Response(WsResponse {
                        request_id,
                        result: individual_result.clone(),
                    });
                    self.broadcast(individual_result.clone());
                    if let Some(who) = self.callbacks.get(&request_id) {
                        self.link.respond(*who, individual_result);
                    };
                }
                self.callbacks.remove(&request_id);
            }
            Message::Reset => {
                self.broadcast(AgentResponse::Disconnected);

                if self.reconnect_timer.is_some() {
                    return;
                }
                self.web_socket_task = None;
                self.ready_for_messages = false;
                self.reconnect_sleep_ms = std::cmp::min(self.reconnect_sleep_ms * 2, 30_000);
                self.reconnect_timer = Some(TimeoutService::spawn(
                    std::time::Duration::from_millis(self.reconnect_sleep_ms as u64),
                    self.link.callback(|_| Message::Initialize),
                ));
            }
        }
    }

    // Handle incoming messages from components of other agents.
    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) {
        match msg {
            AgentMessage::Initialize => {
                if self.web_socket_task.is_none() {
                    self.initialize_websockets();
                }
            }
            AgentMessage::Request(req) => {
                self.ws_send(req, Some(who));
            }
            AgentMessage::Reset => {
                self.update(Message::Reset);
            }
            AgentMessage::RegisterBroadcastHandler => {
                self.broadcasts.insert(who);
            }
            AgentMessage::UnregisterBroadcastHandler => {
                self.broadcasts.remove(&who);
            }
            AgentMessage::LogOut => {
                self.auth_state = AuthState::Unauthenticated;
                self.storage_enabled = false;
                self.save_login_state();
                self.update(Message::Reset);
            }
            AgentMessage::QueryStorageStatus => {
                self.link
                    .respond(who, AgentResponse::StorageStatus(self.storage_enabled));
            }
            AgentMessage::EnableStorage => {
                self.storage_enabled = true;
                self.save_login_state();
                self.link
                    .respond(who, AgentResponse::StorageStatus(self.storage_enabled));
            }
            AgentMessage::DisableStorage => {
                self.storage_enabled = false;
                self.save_login_state();
                self.link
                    .respond(who, AgentResponse::StorageStatus(self.storage_enabled));
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.broadcasts.remove(&id);
        let callbacks_to_remove = self
            .callbacks
            .iter()
            .filter(|(_, handler)| **handler == id)
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();

        for id in callbacks_to_remove {
            self.callbacks.remove(&id);
        }
    }
}

use yew::format::{Binary, Bincode, Text};
#[derive(Debug)]
pub struct WsMessageProxy<T>(pub T);

impl<T> From<Text> for WsMessageProxy<Result<T, anyhow::Error>>
where
    T: Default,
{
    fn from(_: Text) -> Self {
        unreachable!("We shouldn't be getting non-binary messages over our websockets")
    }
}

#[derive(Debug, Error)]
enum WsMessageError {
    #[error("error decoding bincode")]
    Serialization(#[from] Box<bincode::ErrorKind>),
}

impl<T> From<Binary> for WsMessageProxy<Result<T, anyhow::Error>>
where
    for<'de> T: serde::Deserialize<'de>,
{
    fn from(bytes: Binary) -> Self {
        match bytes {
            Ok(bytes) => WsMessageProxy(match bincode::deserialize(bytes.as_slice()) {
                Ok(result) => Ok(result),
                Err(err) => Err(WsMessageError::Serialization(err).into()),
            }),
            Err(err) => Self(Err(err)),
        }
    }
}

impl ApiAgent {
    fn initialize_websockets(&mut self) {
        if self.reconnect_timer.is_some() {
            return;
        }
        let callback = self.link.callback(|WsMessageProxy(msg)| match msg {
            Ok(data) => Message::Message(data),
            Err(_) => Message::Reset,
        });
        let notification = self.link.callback(|status| match status {
            WebSocketStatus::Opened => Message::Connected,
            WebSocketStatus::Closed | WebSocketStatus::Error => Message::Reset,
        });
        self.web_socket_task = Some(
            WebSocketService::connect(&Self::websocket_url(), callback, notification).unwrap(),
        );
    }

    #[cfg(debug_assertions)]
    fn websocket_url() -> &'static str {
        "ws://localhost:7878/v1/ws"
    }
    #[cfg(not(debug_assertions))]
    fn websocket_url() -> &'static str {
        "wss://api.ncog.id/v1/ws"
    }

    fn ws_send(&mut self, request: ServerRequest, who: Option<HandlerId>) {
        self.ws_request_id += 1;
        if self.ready_for_messages {
            if let Some(websocket) = self.web_socket_task.as_mut() {
                if let Some(who) = who {
                    self.callbacks.insert(self.ws_request_id, who);
                }
                websocket.send_binary(Bincode(&WsRequest {
                    id: self.ws_request_id,
                    request,
                }));
            }
        }
    }

    fn broadcast(&self, response: AgentResponse) {
        for entry in self.broadcasts.iter() {
            self.link.respond(*entry, response.clone());
        }
    }

    fn save_login_state(&mut self) {
        if self.storage_enabled {
            self.storage.store(
                "login_state",
                Json(&self.auth_state.encrypted_login_information()),
            );
        } else {
            self.storage.remove("login_state");
            self.storage.remove("return_path");
        }
    }

    fn installation_id(&self) -> Option<Uuid> {
        match &self.auth_state {
            AuthState::PreviouslyAuthenticated(uuid) => Some(*uuid),
            AuthState::Authenticated(state) => Some(state.installation_id),
            AuthState::Unauthenticated => None,
        }
    }

    fn handle_ws_message(&mut self, response: &ServerResponse) {
        trace!("Received response: {:?}", response);
        match response {
            ServerResponse::AdoptInstallationId { installation_id } => {
                self.auth_state = match &self.auth_state {
                    AuthState::Unauthenticated | AuthState::PreviouslyAuthenticated(_) => {
                        AuthState::PreviouslyAuthenticated(*installation_id)
                    }
                    AuthState::Authenticated(_) => unreachable!(
                        "Adopted an installation id even though we were already authenticated"
                    ),
                };
                self.save_login_state();
            }
            ServerResponse::AuthenticateAtUrl { url } => {
                let window = web_sys::window().expect("Need a window");
                window
                    .location()
                    .set_href(url)
                    .expect("Error setting location for redirect");
            }
            ServerResponse::Error { message } => error!("Error from server: {:?}", message),
            ServerResponse::Ping { timestamp, .. } => self.ws_send(
                ServerRequest::Pong {
                    original_timestamp: *timestamp,
                    timestamp: wasm_utc_now().timestamp_millis() as f64 / 1_000_000.0,
                },
                None,
            ),
            ServerResponse::Authenticated { profile, .. } => {
                self.auth_state = AuthState::Authenticated(AuthenticatedState {
                    installation_id: self
                        .installation_id()
                        .expect("Somehow authenticated without an installation_id"),
                    profile: profile.clone(),
                });
                self.save_login_state();

                let window = web_sys::window().expect("Need a window");
                if let Ok(path) = window.location().pathname() {
                    if path.contains("/login") {
                        let mut agent = RouteAgentBridge::new(Callback::noop());
                        agent.send(RouteRequest::ReplaceRoute(Route::new_no_state("/")));
                    }
                }
            }
            _ => {}
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct AuthenticatedState {
    pub installation_id: Uuid,
    pub profile: UserProfile,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
enum AuthState {
    Unauthenticated,
    PreviouslyAuthenticated(Uuid),
    Authenticated(AuthenticatedState),
}

impl AuthState {
    fn encrypted_login_information(&self) -> EncryptedLoginInformation {
        use aead::{generic_array::GenericArray, Aead, NewAead};
        use aes_gcm::Aes256Gcm;

        let key = encryption_key();
        let key = GenericArray::from_exact_iter(key.bytes()).unwrap();
        let aead = Aes256Gcm::new(key);

        let mut rng = thread_rng();
        let key = std::iter::repeat(())
            .map(|()| rng.gen())
            .take(12)
            .collect::<Vec<_>>();
        let nonce = GenericArray::from_slice(&key);
        let payload = serde_json::to_string(&self).expect("Error serializing login state");
        let payload = payload.into_bytes();
        let payload: &[u8] = &payload;
        let ciphertext = aead.encrypt(nonce, payload).expect("encryption failure!");

        EncryptedLoginInformation {
            iv: base64::encode_config(nonce, base64::URL_SAFE_NO_PAD),
            encrypted: base64::encode_config(&ciphertext, base64::URL_SAFE_NO_PAD),
        }
    }
}

#[cfg(not(debug_assertions))]
fn encryption_key() -> &'static str {
    std::env!("NCOG_CLIENT_ENCRYPTION_KEY")
}

#[cfg(debug_assertions)]
fn encryption_key() -> &'static str {
    "pcnhAlQq9VNmOp325GFU8JtR8vuD1wIj"
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct EncryptedLoginInformation {
    pub iv: String,
    pub encrypted: String,
}

impl EncryptedLoginInformation {
    pub fn auth_state(&self) -> AuthState {
        if !self.iv.is_empty() && !self.encrypted.is_empty() {
            if let Ok(nonce) = base64::decode_config(&self.iv, base64::URL_SAFE_NO_PAD) {
                if let Ok(ciphertext) =
                    base64::decode_config(&self.encrypted, base64::URL_SAFE_NO_PAD)
                {
                    use aead::{generic_array::GenericArray, Aead, NewAead};
                    use aes_gcm::Aes256Gcm;

                    let key = encryption_key();
                    let key =
                        GenericArray::from_exact_iter(key.bytes()).expect("Invalid encryption key");
                    let aead = Aes256Gcm::new(key);
                    let nonce = GenericArray::from_exact_iter(nonce).expect("Invalid nonce");
                    let ciphertext: &[u8] = &ciphertext;
                    if let Ok(plaintext) = aead.decrypt(&nonce, ciphertext) {
                        if let Ok(state) = serde_json::from_slice::<AuthState>(&plaintext) {
                            return state;
                        }
                    }
                }
            }
        }
        AuthState::Unauthenticated
    }
}
