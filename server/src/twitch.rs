use crate::{env, SERVER_URL};
use serde::Deserialize;
use std::convert::Infallible;
use url::Url;
use uuid::Uuid;
use warp::{Filter, Rejection};

#[derive(Deserialize)]
struct TwitchCallback {
    code: String,
    state: String,
    scope: String,
}

pub fn callback() -> impl warp::Filter<Extract = (impl warp::Reply,), Error = Rejection> + Copy {
    warp::path!("auth" / "callback" / "twitch")
        .and(warp::query())
        .and_then(|callback: TwitchCallback| async move { callback.respond().await })
}

impl TwitchCallback {
    async fn respond(self) -> Result<impl warp::Reply, Infallible> {
        // TODO bad unwrap
        crate::websockets::login_twitch(self.state.parse().unwrap(), self.code)
            .await
            .unwrap();

        Ok("Yay")
    }
}

pub fn authorization_url(installation_id: Uuid) -> String {
    Url::parse_with_params(
        "https://id.twitch.tv/oauth2/authorize",
        &[
            ("client_id", env("TWITCH_CLIENT_ID")),
            ("scope", "openid".to_owned()),
            ("response_type", "code".to_owned()),
            (
                "redirect_uri",
                format!("{}/v1/auth/callback/twitch", SERVER_URL),
            ),
            ("state", installation_id.to_string()),
            // TODO add NONCE
        ],
    )
    .unwrap()
    .to_string()
}
