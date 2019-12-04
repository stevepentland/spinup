use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use futures::future::join_all;
use reqwest::Client;

use crate::config::{Configuration, FileDownloadDefinition, FileDownloadOperation};
use crate::error::SpinupError;

pub async fn execute_download_operations(config: &Configuration) -> Result<Vec<()>, SpinupError> {
    match &config.file_downloads {
        Some(operations) => join_all(
            operations
                .iter()
                .map(|op| execute_download_operation(op))
                .collect::<Vec<_>>(),
        )
        .await
        .into_iter()
        .collect(),
        None => Ok(vec![]),
    }
}

async fn execute_download_operation(operation: &FileDownloadOperation) -> Result<(), SpinupError> {
    let target = operation.download_target_base().unwrap();
    if !target.exists() && fs::create_dir_all(&target).is_err() {
        return Err(SpinupError::FileDownloadFailed);
    }
    debug!("{:?}", target);
    let client: Client = Client::new();
    join_all(
        operation
            .files
            .iter()
            .map(|fl| download_target(&fl, &target, &client))
            .collect::<Vec<_>>(),
    )
    .await
    .iter()
    .fold(
        Ok(()),
        |c, n| if let Err(e) = n { Err(e.clone()) } else { c },
    )
}

async fn download_target(
    definition: &FileDownloadDefinition,
    base_path: &PathBuf,
    client: &Client,
) -> Result<(), SpinupError> {
    let resp = client.get(&definition.source[..]).send().await;
    debug!("Response: {:?}", resp);
    if let Ok(res) = resp {
        if let Ok(bytes) = res.bytes().await {
            debug!("Got bytes");
            let mut file_path = base_path.clone();
            debug!("Base path: {:?}", file_path);
            file_path.push(&definition.target);
            debug!("Target file path: {:?}", file_path);
            if let Ok(mut file) = File::create(file_path) {
                return file
                    .write_all(&bytes)
                    .map_err(|_| SpinupError::FileDownloadFailed);
            }
        }
    }
    Err(SpinupError::FileDownloadFailed)
}
