use crate::models::file_client::FileClient;
use anyhow::{bail, Result};
use std::path::PathBuf;

pub async fn upload_to_grpc(path: &str, addr: &str, port: &str) -> Result<()> {
    let mut client = FileClient::new(addr, port).await?;

    if let Some(file) = path.split(['/', '\\']).last() {
        let full_path = PathBuf::from(path);
        if let Some(dir) = full_path.parent() {
            match client
                .upload_file(file.to_string(), &PathBuf::from(dir))
                .await
            {
                Ok(_) => {
                    println!("File uploaded successfully");
                    return Ok(());
                }
                Err(e) => bail!("Error : {}", e),
            }
        }
    }
    bail!("Something went wrong with the file path {}", path)
}
