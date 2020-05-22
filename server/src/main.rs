use database::{pg, sqlx};
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use tera::Tera;
use uuid::Uuid;
use warp::http::{header, StatusCode};
use warp::Filter;

mod db;
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

    database::migrations::run_all()
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

    // Create an account if it doesn't exist yet for this installation
    let account_id = if let Some(account_id) = sqlx::query!(
        "SELECT account_id FROM installations WHERE id = $1",
        installation_id
    )
    .fetch_one(&mut tx)
    .await?
    .account_id
    {
        account_id
    } else {
        let account_id = sqlx::query!("INSERT INTO accounts DEFAULT VALUES RETURNING id")
            .fetch_one(&mut tx)
            .await?
            .id;
        sqlx::query!(
            "UPDATE installations SET account_id = $1 WHERE id = $2",
            account_id,
            installation_id
        )
        .execute(&mut tx)
        .await?;
        account_id
    };

    // Create an itchio profile
    sqlx::query!("INSERT INTO itchio_profiles (id, account_id, username, url) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET account_id = $2, username = $3, url = $4 ",
        response.user.id,
        account_id,
        response.user.username,
        response.user.url
    ).execute(&mut tx).await?;

    // Create an oauth_token
    sqlx::query!("INSERT INTO oauth_tokens (account_id, service, access_token) VALUES ($1, $2, $3) ON CONFLICT (account_id, service) DO UPDATE SET access_token = $3",
        account_id,
        "itchio",
        access_token
    ).execute(&mut tx).await?;

    Ok(())
}

fn env(var: &str) -> String {
    std::env::var(var).unwrap()
}
