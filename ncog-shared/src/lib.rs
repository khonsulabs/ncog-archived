use basws_shared::{Version, VersionReq};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub mod iam;
pub mod jwk;
pub mod localization;
pub mod permissions;
pub use fluent_templates;
pub use jsonwebtoken;
use jwk::JwtKey;
use permissions::PermissionSet;

pub fn ncog_protocol_version() -> Version {
    Version::parse("0.0.1").unwrap()
}

pub fn ncog_protocol_version_requirements() -> VersionReq {
    VersionReq::parse("=0.0.1").unwrap()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NcogRequest {
    AuthenticationUrl(OAuthProvider),
    IAM(iam::IAMRequest),
    ListPublicJwtKeys,
    RequestIdentityVerificationToken { nonce: [u8; 32], audience: String },
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
pub struct IdentityVerificationClaims {
    #[serde(rename = "iss")]
    pub issuer: String,
    #[serde(rename = "sub")]
    pub subject: String,
    #[serde(rename = "aud")]
    pub audience: String,
    #[serde(rename = "exp")]
    pub expiration_time: u64,
    #[serde(rename = "iat")]
    pub issuance_time: u64,
    pub nonce: [u8; 32],
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NcogResponse {
    JwtPublicKeys(Vec<JwtKey>),
    IdentityVerificationToken {
        token: String,
    },
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

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub id: i64,
    pub login: Option<String>,
    pub display_name: Option<String>,
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
