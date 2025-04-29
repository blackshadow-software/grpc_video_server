use grpc_video_server::{
    file_upload_to_grpc, run_grpc_video_server, utils::addr::EXAMPLE_SERVER_DIST_PATH,
};

pub mod client;
pub mod models;
pub mod server;
pub mod utils;

#[tokio::main]
async fn main() {
    // let path = "/Users/intishar-564613/Downloads/Nature Sounds.mp4";
    // file_upload_to_grpc(path, SERVER_LOCAL_ADDR, "50057")
    //     .await
    //     .unwrap_or_else(|e| {
    //         eprintln!("Error uploading file: {}", e);
    //     });

    run_grpc_video_server(EXAMPLE_SERVER_DIST_PATH, "50057")
        .await
        .unwrap_or_else(|e| {
            eprintln!("Error running gRPC server: {}", e);
        });
}
