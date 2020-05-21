mod migrations;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    migrations::run_all().await.unwrap();
}