use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use futures::future::join_all;
use reqwest::Client;

use crate::config::{Configuration, FileDownloadDefinition, FileDownloadOperation};
use crate::error::{Error, Result};

pub async fn execute_download_operations(config: &Configuration) -> Result<Vec<()>> {
    match &config.file_downloads {
        Some(operations) => {
            // I don't really like this at the moment, but the super-nested results of this join_all
            // are giving me a hard time at the moment
            let errors = join_all(operations.iter().map(|op| execute_download_operation(op)))
                .await
                .into_iter()
                .filter(Result::is_err)
                .nth(1);

            if let Some(res) = errors {
                res
            } else {
                Ok(vec![])
            }
        }
        None => Ok(vec![]),
    }
}

async fn execute_download_operation(operation: &FileDownloadOperation) -> Result<Vec<()>> {
    let target = operation.download_target_base().unwrap();
    if !target.exists() && fs::create_dir_all(&target).is_err() {
        return Err(Error::from(format!(
            "Unable to create target '{}' and it did not exist",
            target.to_string_lossy()
        )));
    }
    debug!("{:?}", target);
    let client: Client = Client::new();
    join_all(
        operation
            .files
            .iter()
            .map(|fl| download_target(&fl, &target, &client)),
    )
    .await
    .into_iter()
    .collect()
}

async fn download_target(
    definition: &FileDownloadDefinition,
    base_path: &PathBuf,
    client: &Client,
) -> Result<()> {
    let bytes = client
        .get(&definition.source[..])
        .send()
        .await?
        .bytes()
        .await?;

    let mut file_path = base_path.clone();
    debug!("Base path: {:?}", file_path);
    file_path.push(&definition.target);
    debug!("Target file path: {:?}", file_path);
    File::create(file_path).map(|mut file| file.write_all(&bytes).or_else(|e| Err(e.into())))?
}
