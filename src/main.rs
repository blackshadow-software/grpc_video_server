pub mod server;
use crate::server::run_grpc;

#[tokio::main]
async fn main() {
    run_grpc().await.unwrap();
}
