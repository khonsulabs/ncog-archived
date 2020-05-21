use chrono::Utc;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

pub const PROTOCOL_VERSION: &'static str = "0.0.1";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerRequest {
    Authenticate {
        version: String,
        installation_id: Option<Uuid>,
    },
    AuthenticationUrl,
    Update {
        new_inputs: Option<Inputs>,
        x_offset: f32,
        timestamp: f64,
    },
    Pong {
        original_timestamp: f64,
        timestamp: f64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Inputs {
    /// -1.0 to 1.0
    pub horizontal_movement: f32,
    pub interact: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerResponse {
    AdoptInstallationId {
        installation_id: Uuid,
    },
    AuthenticateAtUrl {
        url: String,
    },
    Authenticated {
        profile: UserProfile,
    },
    Error {
        message: Option<String>,
    },
    WorldUpdate {
        timestamp: f64,
        profiles: Vec<UserProfile>,
    },
    Ping {
        timestamp: f64,
        average_roundtrip: f64,
        average_server_timestamp_delta: f64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserProfile {
    pub id: i64,
    pub username: String,
    pub map: i32,
    pub x_offset: f32,
    pub last_update_timestamp: Option<f64>,
    pub horizontal_input: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Installation {
    pub id: Uuid,
    pub account_id: Option<i64>,
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
