use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};
use upload::upload_request::Type;
use upload::upload_service_server::UploadService;
use upload::{UploadRequest, UploadResponse};

pub mod upload {
    tonic::include_proto!("upload");
}
#[derive(Default)]
pub struct MyUploadService {
    directory: Arc<PathBuf>,
}

impl MyUploadService {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory: Arc::new(directory),
        }
    }
}

#[tonic::async_trait]
impl UploadService for MyUploadService {
    async fn upload_file(
        &self,
        request: Request<Streaming<UploadRequest>>,
    ) -> Result<Response<UploadResponse>, Status> {
        let mut request_stream = request.into_inner();
        let directory = Arc::clone(&self.directory);

        let task_handle = tokio::spawn(async move {
            let req = match request_stream.next().await {
                Some(Ok(msg)) => msg,
                Some(Err(e)) => {
                    eprintln!("Stream error: {}", e);
                    return Err(Status::internal("Stream error"));
                }
                None => {
                    eprintln!("Upload stream was empty");
                    return Err(Status::internal("Empty upload stream"));
                }
            };

            let file_name = match req.r#type {
                Some(Type::Name(name)) => name,
                _ => {
                    eprintln!("First message was not file name");
                    return Err(Status::internal("Expected file name first"));
                }
            };

            let mut file_path = directory.as_ref().clone();
            file_path.push(&file_name);

            tokio::fs::create_dir_all(file_path.parent().unwrap())
                .await
                .map_err(|e| {
                    eprintln!("Failed to create directory: {}", e);
                    Status::internal("Failed to create directory")
                })?;

            let mut file_handle = File::create(&file_path).await.map_err(|e| {
                println!(
                    "Failed to create file: {}. the file path is : {:?}",
                    e,
                    file_path.display()
                );
                Status::internal("Failed to create file")
            })?;

            while let Some(chunk_result) = request_stream.next().await {
                match chunk_result {
                    Ok(upload) => match upload.r#type {
                        Some(Type::Chunk(chunk)) => {
                            file_handle.write_all(&chunk).await.map_err(|e| {
                                eprintln!("Failed to write chunk: {}", e);
                                Status::internal("Failed to write file chunk")
                            })?;
                        }
                        _ => {
                            eprintln!("Unexpected message type during upload");
                            return Err(Status::internal("Unexpected message type"));
                        }
                    },
                    Err(e) => {
                        eprintln!("Error receiving chunk: {}", e);
                        return Err(Status::internal("Stream receive error"));
                    }
                }
            }

            file_handle.sync_all().await.map_err(|e| {
                eprintln!("Failed to sync file: {}", e);
                Status::internal("Failed to sync file")
            })?;
            println!("File save completed: {:?}", file_path.display());

            Ok(())
        });

        if let Err(err) = task_handle.await.unwrap() {
            println!("Failed to save file: {}", err);
            Err(Status::internal("Failed to save file"))
        } else {
            Ok(Response::new(UploadResponse::default()))
        }
    }
}
