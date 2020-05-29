use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{
    websockets::{WsBatchResponse, WsRequest, WsResponse},
    ServerRequest,
};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use yew::format::Json;
use yew::services::{
    storage::{Area, StorageService},
    timeout::{TimeoutService, TimeoutTask},
    websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
};
use yew::worker::*;

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
}

pub struct ApiAgent {
    link: AgentLink<Self>,
    web_socket_service: WebSocketService,
    web_socket_task: Option<WebSocketTask>,
    ws_request_id: i64,
    timeout: TimeoutService,
    reconnect_timer: Option<TimeoutTask>,
    reconnect_sleep_ms: u32,
    callbacks: HashMap<i64, HandlerId>,
    broadcasts: HashSet<HandlerId>,
    ready_for_messages: bool,
    storage: StorageService,
    return_path: Option<String>,
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
}

impl Agent for ApiAgent {
    type Reach = Context; // Spawn only one instance on the main thread (all components can share this agent)
    type Message = Message;
    type Input = AgentMessage;
    type Output = AgentResponse;

    // Create an instance with a link to the agent.
    fn create(link: AgentLink<Self>) -> Self {
        let mut storage =
            StorageService::new(Area::Session).expect("Error accessing storage service");
        let Json(login_state) = storage.restore("login_state");
        let auth_state = login_state
            .unwrap_or(EncryptedLoginInformation::default())
            .auth_state();
        let Json(return_path) = storage.restore("return_path");
        let return_path = return_path.unwrap_or(None);
        storage.remove("return_path");
        Self {
            link,
            web_socket_service: WebSocketService::new(),
            web_socket_task: None,
            ws_request_id: 0,
            reconnect_sleep_ms: DEFAULT_RECONNECT_TIMEOUT,
            timeout: TimeoutService::new(),
            reconnect_timer: None,
            callbacks: HashMap::new(),
            broadcasts: HashSet::new(),
            ready_for_messages: false,
            storage,
            return_path,
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
                for entry in self.broadcasts.iter() {
                    self.link.respond(*entry, AgentResponse::Connected);
                }
            }
            Message::Message(ws_response) => {
                let request_id = ws_response.request_id;
                for individual_result in ws_response.results.iter() {
                    let individual_result = AgentResponse::Response(WsResponse {
                        request_id,
                        result: individual_result.clone(),
                    });
                    for entry in self.broadcasts.iter() {
                        self.link.respond(*entry, individual_result.clone());
                    }
                    if let Some(who) = self.callbacks.get(&request_id) {
                        self.link.respond(*who, individual_result);
                    };
                }
                self.callbacks.remove(&request_id);
            }
            Message::Reset => {
                if self.reconnect_timer.is_some() {
                    return;
                }
                self.web_socket_task = None;
                self.ready_for_messages = false;
                self.reconnect_sleep_ms = std::cmp::min(self.reconnect_sleep_ms * 2, 30_000);
                self.reconnect_timer = Some(self.timeout.spawn(
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
                self.ws_send(req, who);
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
            self.web_socket_service
                .connect(&Self::websocket_url(), callback, notification)
                .unwrap(),
        );
    }

    #[cfg(debug_assertions)]
    fn websocket_url() -> &'static str {
        "ws://localhost:7878/api/ws"
    }
    #[cfg(not(debug_assertions))]
    fn websocket_url() -> &'static str {
        "wss://ncog.link/api/ws"
    }

    fn ws_send(&mut self, request: ServerRequest, who: HandlerId) {
        self.ws_request_id += 1;
        if self.ready_for_messages {
            if let Some(websocket) = self.web_socket_task.as_mut() {
                self.callbacks.insert(self.ws_request_id, who);
                websocket.send_binary(Bincode(&WsRequest {
                    id: self.ws_request_id,
                    request,
                }));
            }
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PendingAuthorizationState {
    pub code_verifier: String,
    pub state: String,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AuthenticatedState {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
}

use rand::Rng;
use std::iter;
impl PendingAuthorizationState {
    pub fn random() -> Self {
        let mut rng = rand::rngs::OsRng;
        PendingAuthorizationState {
            state: iter::repeat(())
                .map(|()| rng.sample(rand::distributions::Alphanumeric))
                .take(32)
                .collect(),
            code_verifier: base64::encode_config(
                &iter::repeat(())
                    .map(|()| rng.sample(rand::distributions::Alphanumeric))
                    .take(32)
                    .collect::<String>(),
                base64::URL_SAFE_NO_PAD,
            ),
        }
    }
    pub fn code_challenge(&self) -> String {
        base64_sha256(&self.code_verifier)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum AuthState {
    Unauthenticated,
    PreviouslyAuthenticated,
    PendingAuthentication(PendingAuthorizationState),
    Authenticated(AuthenticatedState),
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct EncryptedLoginInformation {
    pub iv: String,
    pub encrypted: String,
}

impl EncryptedLoginInformation {
    pub fn auth_state(&self) -> AuthState {
        if self.iv.len() > 0 && self.encrypted.len() > 0 {
            if let Ok(nonce) = base64::decode_config(&self.iv, base64::URL_SAFE_NO_PAD) {
                if let Ok(ciphertext) =
                    base64::decode_config(&self.encrypted, base64::URL_SAFE_NO_PAD)
                {
                    use aead::{generic_array::GenericArray, Aead, NewAead};
                    use aes_gcm::Aes256Gcm;

                    let key = std::option_env!("NCOG_CLIENT_ENCRYPTION_KEY")
                        .unwrap_or("pcnhAlQq9VNmOp325GFU8JtR8vuD1wIj")
                        .to_owned();
                    let key = GenericArray::from_exact_iter(key.bytes().into_iter())
                        .expect("Invalid encryption key");
                    let aead = Aes256Gcm::new(key);
                    let nonce =
                        GenericArray::from_exact_iter(nonce.into_iter()).expect("Invalid nonce");
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

pub fn base64_sha256(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.input(input);
    let result = hasher.result();
    base64::encode_config(result.as_slice(), base64::URL_SAFE_NO_PAD)
}
