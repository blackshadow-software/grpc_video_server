use client::client::upload_to_grpc;

pub mod client;
pub mod models;
pub mod server;
pub mod utils;

pub mod client_;
pub mod server_;
use anyhow::Result;
use server::server::run_grpc_server;

pub async fn run_grpc_video_server(dist: &str, port: &str) -> Result<()> {
    run_grpc_server(dist, port).await
}

pub async fn file_upload_to_grpc(path: &str, addr: &str, port: &str) -> Result<()> {
    upload_to_grpc(path, addr, port).await
}
