use crate::{api_server_base_url, database, env, webserver_base_url};
use chrono::{NaiveDateTime, Utc};
use ncog_migrations::{pg, sqlx};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::str::FromStr;
use url::Url;
use uuid::Uuid;
use warp::{Filter, Rejection};

#[derive(Deserialize)]
struct TwitchCallback {
    code: String,
    state: String,
    // scope: String,
}

pub fn callback_uri() -> String {
    api_server_base_url()
        .path_and_query("/v1/auth/callback/twitch")
        .build()
        .unwrap()
        .to_string()
}

pub fn callback() -> impl warp::Filter<Extract = (impl warp::Reply,), Error = Rejection> + Copy {
    warp::path!("auth" / "callback" / "twitch")
        .and(warp::query())
        .and_then(|callback: TwitchCallback| async move { callback.respond().await })
}

impl TwitchCallback {
    async fn respond(self) -> Result<impl warp::Reply, Infallible> {
        // TODO bad unwrap
        login_twitch(self.state.parse().unwrap(), self.code)
            .await
            .unwrap();

        Ok(warp::redirect::redirect(
            webserver_base_url().path_and_query("/").build().unwrap(),
        ))
    }
}

pub fn authorization_url(installation_id: Uuid) -> String {
    Url::parse_with_params(
        "https://id.twitch.tv/oauth2/authorize",
        &[
            ("client_id", env("TWITCH_CLIENT_ID")),
            ("scope", "openid".to_owned()),
            ("response_type", "code".to_owned()),
            ("redirect_uri", callback_uri()),
            ("state", installation_id.to_string()),
            // TODO add NONCE
        ],
    )
    .unwrap()
    .to_string()
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

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtKeys {
    pub keys: Vec<JwtKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtKey {
    #[serde(rename = "alg")]
    pub algorithm: String,
    #[serde(rename = "kid")]
    pub key_id: String,
    #[serde(rename = "kty")]
    pub key_type: String,
    #[serde(rename = "e")]
    pub rsa_e: String,
    #[serde(rename = "n")]
    pub rsa_n: String,
    #[serde(rename = "use")]
    pub public_use: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    #[serde(rename = "iss")]
    pub issuer: Option<String>,
    #[serde(rename = "sub")]
    pub subject: Option<String>,
    #[serde(rename = "aud")]
    pub audience: Option<String>,
    #[serde(rename = "exp")]
    pub expiration_time: Option<u64>,
    #[serde(rename = "iat")]
    pub issuance_time: Option<u64>,
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
            ("redirect_uri", callback_uri()),
        ])
        .send()
        .await?
        .json()
        .await?;

    let jwt_keys: JwtKeys = client
        .get("https://id.twitch.tv/oauth2/keys")
        .send()
        .await?
        .json()
        .await?;
    let (algorithm, jwt_key) = jwt_keys
        .keys
        .into_iter()
        .filter_map(|key| {
            jsonwebtoken::Algorithm::from_str(&key.algorithm)
                .map(|algorithm| (algorithm, key))
                .ok()
        })
        .find(|(_, key)| key.key_type == "RSA")
        .expect("Twitch has no RS256 keys");
    let jwt_key = jsonwebtoken::DecodingKey::from_rsa_components(&jwt_key.rsa_n, &jwt_key.rsa_e);

    let validation = jsonwebtoken::Validation::new(algorithm);
    let token = jsonwebtoken::decode::<JwtClaims>(&tokens.id_token, &jwt_key, &validation)?;

    let expiration_time = NaiveDateTime::from_timestamp(
        token
            .claims
            .expiration_time
            .ok_or_else(|| anyhow::anyhow!("jwt missing expiration"))? as i64,
        0,
    );
    if token.claims.issuer != Some("https://id.twitch.tv/oauth2".to_owned())
        || expiration_time < Utc::now().naive_utc()
    {
        anyhow::bail!("Invalid JWT Token");
    }

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
