use std::convert::Infallible;
use std::path::Path;
use warp::{Filter, Reply};

mod database;
mod pubsub;
// mod randomnames;
mod websockets;

#[macro_use]
extern crate slog_scope;

#[cfg(debug_assertions)]
const SERVER_URL: &'static str = "http://localhost:7878";
#[cfg(not(debug_assertions))]
const SERVER_URL: &'static str = "https://ncog.link";

#[cfg(debug_assertions)]
const STATIC_FOLDER_PATH: &'static str = "../webapp/static";
#[cfg(not(debug_assertions))]
const STATIC_FOLDER_PATH: &'static str = "static";

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Error initializing environment");
    let _log_guard = initialize_logging();

    let base_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_owned());
    let base_dir = Path::new(&base_dir);
    migrations::run_all()
        .await
        .expect("Error running migrations");

    websockets::initialize().await;

    tokio::spawn(pubsub::pg_notify_loop());

    let healthcheck = warp::get()
        .and(warp::path("__healthcheck"))
        .and_then(healthcheck);

    let static_path = base_dir.join(STATIC_FOLDER_PATH);
    let index_path = static_path.join("index.html");
    let spa = warp::get().and(warp::fs::dir(static_path).or(warp::fs::file(index_path)));

    let websockets = warp::path!("ws")
        .and(warp::path::end())
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(|websocket| websockets::main(websocket)));

    let custom_logger = warp::log::custom(|info| {
        if info.status().is_server_error() {
            error!("Request Served"; "path" => info.path(), "method" => info.method().as_str(), "status" => info.status().as_str());
        } else {
            info!("Request Served"; "path" => info.path(), "method" => info.method().as_str(), "status" => info.status().as_u16());
        }
    });

    let api = warp::path("api").and(websockets);
    let routes = healthcheck
        .or(api)
        .or(spa)
        .with(custom_logger)
        .with(warp::reply::with::header(
            "Access-Control-Allow-Origin",
            "*",
        ))
        .with(warp::reply::with::header(
            "Access-Control-Allow-Headers",
            "Authorization, *",
        ));
    info!("Starting listening on port 7878");
    warp::serve(routes).run(([0, 0, 0, 0], 7878)).await;
}

async fn healthcheck() -> Result<impl Reply, Infallible> {
    Ok("ok")
}

fn env(var: &str) -> String {
    std::env::var(var).unwrap()
}

use slog::{o, Drain};
pub fn initialize_logging() -> slog_scope::GlobalLoggerGuard {
    let json = slog_json::Json::new(std::io::stdout())
        .add_default_keys()
        .build()
        .fuse();
    let async_json = slog_async::Async::new(json).build().fuse();
    let log = slog::Logger::root(async_json, o!());
    slog_scope::set_global_logger(log)
}
