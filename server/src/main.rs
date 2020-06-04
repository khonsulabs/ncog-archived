use std::convert::Infallible;
use std::path::Path;
use tokio::io::AsyncReadExt;
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
    info!("server starting up");

    let base_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_owned());
    let base_dir = Path::new(&base_dir);
    info!("Running migrations");
    migrations::run_all()
        .await
        .expect("Error running migrations");

    info!("Done running migrations");
    websockets::initialize().await;

    tokio::spawn(pubsub::pg_notify_loop());

    let healthcheck = warp::get()
        .and(warp::path("__healthcheck"))
        .and_then(healthcheck);

    let static_path = base_dir.join(STATIC_FOLDER_PATH);
    let index_path = static_path.join("index.html");

    let spa = warp::get().and(warp::fs::dir(static_path).or(warp::fs::file(index_path)));

    // In release mode, we want to proxy with s3, but when debugging we want to use the current build.
    #[cfg(not(debug_assertions))]
    let spa = {
        let s3_proxy = warp::path!("pkg" / String).and_then(|file_name| s3_proxy(file_name));
        s3_proxy.or(spa)
    };

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

use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, S3Client, S3};
use warp::http::StatusCode;
async fn s3_proxy(file_name: String) -> Result<impl Reply, Infallible> {
    let client = S3Client::new(Region::UsEast1);
    let mime = mime_guess::from_path(&file_name)
        .first_or_text_plain()
        .to_string();

    match client
        .get_object(GetObjectRequest {
            bucket: "ncog-releasesbucket-hvnlnyp4xejx".to_owned(), // TODO make this an environment variable
            key: format!("pkg/{}", file_name),
            ..Default::default()
        })
        .await
    {
        Ok(response) => {
            let mut bytes = Vec::new();
            response
                .body
                .unwrap()
                .into_async_read()
                .read_to_end(&mut bytes)
                .await
                .expect("Error reading from s3");

            Ok(warp::reply::with_status(
                warp::reply::with_header(bytes, warp::http::header::CONTENT_TYPE, mime),
                StatusCode::OK,
            ))
        }
        Err(err) => {
            error!("Error proxying from s3: {}", err);
            let empty = Vec::new();
            Ok(warp::reply::with_status(
                warp::reply::with_header(empty, warp::http::header::CONTENT_TYPE, mime),
                StatusCode::NOT_FOUND,
            ))
        }
    }
}
