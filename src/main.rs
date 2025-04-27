use grpc_video_server::server::server::run_grpc;

pub mod models;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() {
    run_grpc().await.unwrap();
}
