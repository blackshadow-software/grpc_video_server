use crate::models::file_server::upload::upload_request::Type;
use crate::models::file_server::upload::upload_service_server::UploadServiceServer;
use crate::models::file_server::upload::UploadRequest;
use crate::models::file_server::MyUploadService;
use crate::utils::addr::*;
use anyhow::{bail, Result};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use tonic::transport::Server;
use tonic::Status;

pub async fn run_grpc_server(dist: &str, port: &str) -> Result<()> {
    let addr = format!("{}:{}", SERVER_LOCAL_ADDR, port);
    let addr = SocketAddr::from_str(&addr).unwrap();
    let path: PathBuf = PathBuf::from_str(dist)?;
    let file_service = MyUploadService::new(path);
    let service = UploadServiceServer::new(file_service);
    println!("Starting gRPC server on {}", addr);
    Server::builder().add_service(service).serve(addr).await?;

    Ok(())
}

pub trait TypeExt {
    fn get_type(self) -> anyhow::Result<Type>;
    fn file_name(self) -> anyhow::Result<String>;
}

impl TypeExt for Option<Result<UploadRequest, Status>> {
    fn get_type(self) -> anyhow::Result<Type> {
        if self.is_some() {
            return Ok(self.unwrap()?.r#type.unwrap());
        }
        bail!("Wrong message type")
    }

    fn file_name(self) -> anyhow::Result<String> {
        if self.clone().get_type().is_ok() {
            match self.get_type().unwrap() {
                Type::Name(name) => return Ok(name),
                _ => {
                    bail!("Wrong message type")
                }
            }
        }
        bail!("Wrong message type")
    }
}
