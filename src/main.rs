use gourami_social::routes::run_server;

#[tokio::main]
async fn main() {
    run_server().await;
}
