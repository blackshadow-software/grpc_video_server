use crate::models::file::FileServiceImpl;
use crate::server_::MyGreeter;
use crate::utils::addr::*;
use anyhow::Result;
use file_protos::file_service_server::FileServiceServer;
use hello_world::greeter_server::GreeterServer;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use tonic::transport::Server;
pub mod file_protos {
    tonic::include_proto!("file_protos");
}

pub async fn run_grpc() -> Result<()> {
    let addr = format!("{}:{}", GRPC_ADDR, GRPC_PORT);
    let addr = SocketAddr::from_str(&addr).unwrap();
    let path = PathBuf::from_str(FILE_PATH)?;
    let file_service = FileServiceImpl::new(path);
    let service = FileServiceServer::new(file_service);
    let greeter = MyGreeter::default();
    Server::builder().add_service(service).serve(addr).await?;

    Ok(())
}
