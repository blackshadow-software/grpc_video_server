use grpc_video_server::{file_upload_to_grpc, run_grpc_video_server};
use utils::addr::GRPC_ADDR;

pub mod client;
pub mod models;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() {
    let path = "/Users/intishar-564613/Downloads/Nature Sounds.mp4";
    file_upload_to_grpc(path, GRPC_ADDR, "50057")
        .await
        .unwrap_or_else(|e| {
            eprintln!("Error uploading file: {}", e);
        });
    // run_grpc_video_server(
    //     "/Users/intishar-564613/Desktop/untitled folder/VIDEOS",
    //     "50057",
    // )
    // .await
    // .unwrap_or_else(|e| {
    //     eprintln!("Error running gRPC server: {}", e);
    // });
}
