#![type_length_limit = "8273194"]
#[macro_use]
extern crate tracing;
use std::convert::Infallible;
use std::path::Path;
use tracing_subscriber::prelude::*;
use warp::{Filter, Reply};

pub mod database;
mod pubsub;
mod twitch;
// mod randomnames;
mod websockets;

#[cfg(debug_assertions)]
fn webserver_base_url() -> warp::http::uri::Builder {
    warp::http::uri::Uri::builder()
        .scheme("http")
        .authority("localhost:7879")
}
#[cfg(not(debug_assertions))]
fn webserver_base_url() -> warp::http::uri::Builder {
    warp::http::uri::Uri::builder()
        .scheme("https")
        .authority("ncog.id")
}

#[cfg(debug_assertions)]
fn api_server_base_url() -> warp::http::uri::Builder {
    warp::http::uri::Uri::builder()
        .scheme("http")
        .authority("localhost:7878")
}
#[cfg(not(debug_assertions))]
fn api_server_base_url() -> warp::http::uri::Builder {
    warp::http::uri::Uri::builder()
        .scheme("https")
        .authority("api.ncog.id")
}

#[cfg(debug_assertions)]
const STATIC_FOLDER_PATH: &str = "../ncog-web/static";
#[cfg(not(debug_assertions))]
const STATIC_FOLDER_PATH: &str = "static";

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Error initializing environment");
    initialize_logging();
    info!("server starting up");

    let base_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_owned());
    let base_dir = Path::new(&base_dir);
    info!("Running migrations");
    ncog_migrations::run_all()
        .await
        .expect("Error running migrations");

    info!("Done running migrations");
    let websocket_server = websockets::initialize();
    let notify_server = websocket_server.clone();

    tokio::spawn(async {
        pubsub::pg_notify_loop(notify_server)
            .await
            .expect("Error on pubsub thread")
    });

    let healthcheck = warp::get()
        .and(warp::path("__healthcheck"))
        .and_then(healthcheck);

    let static_path = base_dir.join(STATIC_FOLDER_PATH);
    let index_path = static_path.join("index.html");

    let websocket_route = warp::path!("ws")
        .and(warp::path::end())
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let websocket_server = websocket_server.clone();
            ws.on_upgrade(|ws| async move { websocket_server.incoming_connection(ws).await })
        });

    let custom_logger = warp::log::custom(|info| {
        if info.status().is_server_error() {
            error!(
                path = info.path(),
                method = info.method().as_str(),
                status = info.status().as_str(),
                "Request Served"
            );
        } else {
            info!(
                path = info.path(),
                method = info.method().as_str(),
                status = info.status().as_u16(),
                "Request Served"
            );
        }
    });

    let auth = twitch::callback();

    let api = warp::path("v1").and(websocket_route.or(auth));
    let routes = healthcheck
        .or(api)
        .with(custom_logger)
        .with(warp::reply::with::header(
            "Access-Control-Allow-Origin",
            "*",
        ))
        .with(warp::reply::with::header(
            "Access-Control-Allow-Headers",
            "Authorization, *",
        ));

    let main_server = warp::serve(routes).run(([0, 0, 0, 0], 7878));

    let spa = warp::get()
        .and(warp::fs::dir(static_path).or(warp::fs::file(index_path)))
        .with(custom_logger);
    let spa_only_server = warp::serve(spa).run(([0, 0, 0, 0], 7879));

    info!("Starting listening on port 7878 (api) and 7879 (spa)");
    tokio::join!(main_server, spa_only_server);
}

async fn healthcheck() -> Result<impl Reply, Infallible> {
    Ok("ok")
}

fn env(var: &str) -> String {
    std::env::var(var).unwrap()
}

pub fn initialize_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::EnvFilter::default())
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE),
        )
        .try_init()
        .unwrap()
}
