use file_protos::{
    file_service_server::FileService, upload_file_request::Type, DownloadFileRequest,
    DownloadFileResponse, ListFilesRequest, ListFilesResponse, UploadFileRequest,
    UploadFileResponse,
};
pub mod file_protos {
    tonic::include_proto!("file_protos");
}

use anyhow::bail;
use std::{path::PathBuf, sync::Arc};
use tokio::{
    fs::{read_dir, File},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::{self},
};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

#[derive(Default)]
pub struct FileServiceImpl {
    directory: Arc<PathBuf>,
}

impl FileServiceImpl {
    const CHANNEL_SIZE: usize = 10;
    const CHUNK_SIZE_BYTES: u64 = 1024 * 1024; // 1 MB

    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory: Arc::new(directory),
        }
    }
}

#[tonic::async_trait]
impl FileService for FileServiceImpl {
    type DownloadFileStream = ReceiverStream<Result<DownloadFileResponse, Status>>;
    type ListFilesStream = ReceiverStream<Result<ListFilesResponse, Status>>;

    // #[instrument(skip(self))]
    async fn download_file(
        &self,
        request: Request<DownloadFileRequest>,
    ) -> Result<Response<Self::DownloadFileStream>, Status> {
        let request = request.into_inner();
        let (tx, rx) = mpsc::channel(Self::CHANNEL_SIZE);
        let tx_error = tx.clone();
        let directory = Arc::clone(&self.directory);

        let mut file_path = PathBuf::new();
        file_path.push(directory.as_ref());
        file_path.push(request.name);

        tokio::spawn(async move {
            let result = async move {
                let file = File::open(file_path).await?;
                let mut handle = file.take(Self::CHUNK_SIZE_BYTES);

                loop {
                    let mut response = DownloadFileResponse {
                        chunk: Vec::with_capacity(Self::CHUNK_SIZE_BYTES as usize),
                    };

                    let n = handle.read_to_end(&mut response.chunk).await?;

                    if 0 == n {
                        break;
                    } else {
                        handle.set_limit(Self::CHUNK_SIZE_BYTES);
                    }

                    if let Err(err) = tx.send(Ok(response)).await {
                        println!("Failed to send file chunk: {}", err);
                        break;
                    }

                    if n < Self::CHUNK_SIZE_BYTES as usize {
                        break;
                    }
                }

                Ok::<(), anyhow::Error>(())
            }
            .await;

            if let Err(err) = result {
                println!("Failed to download file: {}", err);
                let send_result = tx_error
                    .send(Err(Status::internal("Failed to send file")))
                    .await;

                if let Err(err) = send_result {
                    println!("Failed to send error: {}", err);
                } else {
                    println!("Sent error to client");
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
    // #[instrument(skip(self))]
    async fn upload_file(
        &self,
        request: Request<Streaming<UploadFileRequest>>,
    ) -> Result<Response<UploadFileResponse>, Status> {
        let mut request_stream = request.into_inner();
        let directory = Arc::clone(&self.directory);

        let task_handle = tokio::spawn(async move {
            let mut file_name = "".to_string();
            let req = request_stream.next().await.clone();
            let file_type = req.clone().get_type();
            let file_type_name = req.clone().file_name();
            if file_type.is_err() || file_type_name.is_err() {
                return Err(Status::internal("Wrong message type"));
            } else {
                file_name = file_type_name.unwrap()
            }

            let mut file_path = PathBuf::new();
            file_path.push(directory.as_ref());
            file_path.push(&file_name);

            let mut file_handle = File::create(file_path).await?;

            while let Some(file_upload) = request_stream.next().await.clone() {
                match file_upload?.r#type {
                    Some(Type::Chunk(chunk)) => {
                        file_handle.write_all(&chunk).await?;
                    }
                    _ => {
                        println!("Wrong message type");
                    }
                }
            }
            file_handle.sync_all().await?;
            Ok(())
        });

        if let Err(err) = task_handle.await.unwrap() {
            println!("Failed to upload file: {}", err);
            Err(Status::internal("Failed to upload file"))
        } else {
            Ok(Response::new(UploadFileResponse::default()))
        }
    }
    // #[instrument(skip(self))]
    async fn list_files(
        &self,
        _request: Request<ListFilesRequest>,
    ) -> Result<Response<Self::ListFilesStream>, Status> {
        let (tx, rx) = mpsc::channel(Self::CHANNEL_SIZE);
        let directory = Arc::clone(&self.directory);
        let tx_error = tx.clone();

        tokio::spawn(async move {
            let result = async move {
                let mut dir_stream = read_dir(directory.as_ref()).await?;

                while let Some(dir_entry) = dir_stream.next_entry().await? {
                    let file_metadata = dir_entry.metadata().await?;
                    if !file_metadata.is_file() {
                        continue;
                    }

                    let file_name = dir_entry.file_name().to_string_lossy().to_string();
                    println!("File name: {}", file_name);
                    let file_size = file_metadata.len();

                    if let Err(err) = tx
                        .send(Ok(ListFilesResponse {
                            name: file_name,
                            size: file_size,
                        }))
                        .await
                    {
                        println!("Failed to send file name: {}", err);
                        break;
                    }
                }

                Ok::<(), anyhow::Error>(())
            }
            .await;

            if let Err(err) = result {
                println!("Failed to list files: {}", err);
                let send_result = tx_error
                    .send(Err(Status::internal("Failed to list files")))
                    .await;

                if let Err(err) = send_result {
                    println!("Failed to send error: {}", err);
                } else {
                    println!("Sent error to client");
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

pub trait TypeExt {
    fn get_type(self) -> anyhow::Result<Type>;
    fn file_name(self) -> anyhow::Result<String>;
}

impl TypeExt for Option<Result<UploadFileRequest, Status>> {
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
