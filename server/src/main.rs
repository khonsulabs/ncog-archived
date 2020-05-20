use warp::Filter;

#[tokio::main]
async fn main() {
    let routes = warp::any().map(|| "Hello, World!");
    warp::serve(routes).run(([0, 0, 0, 0], 7878)).await;
}
