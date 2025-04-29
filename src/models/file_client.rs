use anyhow::Result;
use std::{net::IpAddr, path::PathBuf};
use tokio::{fs, io::AsyncReadExt, sync::mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::channel::Channel;

use super::file_server::upload::upload_service_client::UploadServiceClient;
use crate::{
    models::file_server::upload::{upload_request, UploadRequest},
    utils::addr::{CHANNEL_SIZE, CHUNK_SIZE_BYTES},
};

#[derive(Clone)]
pub struct FileClient<T> {
    client: UploadServiceClient<T>,
}

fn create_uri(host: &str, port: &str, tls: bool) -> String {
    let scheme = if tls { "https" } else { "http" };

    if let Ok(ip_addr) = host.parse::<IpAddr>() {
        return match ip_addr {
            IpAddr::V4(ipv4) => format!("{scheme}://{ipv4}:{port}"),
            IpAddr::V6(ipv6) => format!("{scheme}://[{ipv6}]:{port}"),
        };
    }

    format!("{scheme}://{host}:{port}")
}

impl FileClient<Channel> {
    pub async fn new(address: &str, port: &str) -> Result<Self> {
        let dst = create_uri(address, port, false);
        println!("Connecting to {}", dst);

        let endpoint = Channel::from_shared(dst)?;
        let channel = endpoint.connect().await?;
        let client = UploadServiceClient::new(channel);
        println!("Connected");
        Ok(Self { client })
    }

    pub async fn upload_file(&mut self, file: String, directory: &PathBuf) -> Result<()> {
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
        let receiver_stream = ReceiverStream::new(rx);

        let mut file_path = PathBuf::new();
        file_path.push(&directory);
        file_path.push(&file);

        println!(
            "Exists: {:?}, File path: {:?}",
            file_path.exists(),
            file_path
        );

        let task_handle = tokio::spawn(async move {
            if let Err(err) = tx
                .send(UploadRequest {
                    r#type: Some(upload_request::Type::Name(file)),
                })
                .await
            {
                println!("{}", err);
                Err(err)?;
            }

            let file = fs::File::open(file_path).await?;
            let mut handle = file.take(CHUNK_SIZE_BYTES);

            println!("Reading chunk");
            loop {
                let mut chunk = Vec::with_capacity(CHUNK_SIZE_BYTES as usize);

                let n = handle.read_to_end(&mut chunk).await?;
                if 0 == n {
                    break;
                } else {
                    handle.set_limit(CHUNK_SIZE_BYTES);
                }

                let request = UploadRequest {
                    r#type: Some(upload_request::Type::Chunk(chunk)),
                };

                if let Err(err) = tx.send(request).await {
                    println!("tx send error : {}", err);
                    Err(err)?;
                }

                if n < CHUNK_SIZE_BYTES as usize {
                    break;
                }
            }
            println!("Chunk sent");
            Ok::<(), anyhow::Error>(())
        });

        match self.client.upload_file(receiver_stream).await {
            Ok(r) => println!("File uploaded successfully : {:?}", r),
            Err(e) => println!("Error uploading file: {}", e),
        }

        if let Err(err) = task_handle.await? {
            println!("task handler error : {}", err);
            Err(err)?;
        }

        Ok(())
    }
}
