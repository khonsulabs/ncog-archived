use chrono::Utc;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

pub mod iam;
pub mod localization;
pub mod permissions;
pub mod websockets;
pub use fluent_templates;
use permissions::PermissionSet;
use websockets::WsBatchResponse;

pub const PROTOCOL_VERSION: &str = "0.0.1";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerRequest {
    AuthenticationUrl(OAuthProvider),
    IAM(iam::IAMRequest),
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum OAuthProvider {
    Twitch,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Inputs {
    /// -1.0 to 1.0
    pub horizontal_movement: f32,
    pub interact: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerResponse {
    AuthenticateAtUrl {
        url: String,
    },
    Authenticated {
        profile: UserProfile,
        permissions: PermissionSet,
    },
    Unauthenticated,
    Error {
        message: Option<String>,
    },
    IAM(iam::IAMResponse),
}

impl ServerResponse {
    pub fn into_ws_response(self, request_id: i64) -> WsBatchResponse {
        WsBatchResponse {
            request_id,
            results: vec![self],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub id: i64,
    pub screenname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Installation {
    pub id: Uuid,
    pub account_id: Option<i64>,
    pub nonce: Option<Vec<u8>>,
    pub private_key: Option<Vec<u8>>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum NpcModel {
    GreenGuy = 1,
    OrangeGuy = 3,
}

pub const WALK_SPEED: f32 = 32.0;

pub fn current_timestamp() -> f64 {
    Utc::now().timestamp_nanos() as f64 / 1_000_000_000.0
}

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
