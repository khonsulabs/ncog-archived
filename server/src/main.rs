use lazy_static::lazy_static;
use migrations::{pg, sqlx};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use tera::Tera;
use uuid::Uuid;
use warp::http::{header, StatusCode};
use warp::Filter;

mod pubsub;
mod websockets;

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template("logged_in", include_str!("logged_in.html"))
            .unwrap();
        tera
    };
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Error initializing environment");

    migrations::run_all()
        .await
        .expect("Error running migrations");

    tokio::spawn(pubsub::pg_notify_loop());

    let websockets = warp::path!("ws")
        .and(warp::path::end())
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(|websocket| websockets::main(websocket)));
    // let client_authorize = warp::path!("auth" / "client").map(|| oauth_client_authenticate());
    let itchio_callback = warp::path!("auth" / "itchio_callback").map(|| itchio_callback());
    let receive_token = warp::path!("auth" / "receive_token")
        .and(warp::body::form())
        .map(|body: HashMap<String, String>| {
            receive_token(&body["state"], body["access_token"].clone())
        });
    let oauth = itchio_callback.or(receive_token);
    let routes = websockets
        .or(oauth)
        .or(warp::any().map(|| warp::reply::with_status("Not Found", StatusCode::NOT_FOUND)));

    warp::serve(routes).run(([0, 0, 0, 0], 7878)).await;
}

fn itchio_callback() -> impl warp::reply::Reply {
    let mut context = tera::Context::new();
    context.insert("post_url", "http://localhost:7878/auth/receive_token");

    warp::reply::with_header(
        TEMPLATES.render("logged_in", &context).unwrap(),
        header::CONTENT_TYPE,
        "text/html; charset=UTF-8",
    )
}

fn receive_token(state: &str, access_token: String) -> impl warp::reply::Reply {
    let installation_id = match Uuid::parse_str(state) {
        Ok(uuid) => uuid,
        Err(_) => {
            println!("Invalid UUID in state");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    tokio::spawn(async move {
        login_itchio(installation_id, access_token)
            .await
            .expect("Error logging into itchio")
    });
    StatusCode::OK
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

async fn login_itchio(installation_id: Uuid, access_token: String) -> Result<(), anyhow::Error> {
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
    let mut tx = pg.begin().await?;
    let account = sqlx::query!(
        "SELECT account_lookup($1, $2) as account_id",
        response.user.id,
        response.user.username
    )
    .fetch_one(&mut tx)
    .await
    .expect("Function should always return a value");

    sqlx::query!(
        "SELECT installation_login($1, $2, $3) as rows_changed",
        installation_id,
        account.account_id,
        access_token,
    )
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;

    Ok(())
}

fn env(var: &str) -> String {
    std::env::var(var).unwrap()
}
