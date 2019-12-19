//! The file_downloads module handles downloading various files
//! and running other related commands.

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use futures::future::join_all;
use reqwest::Client;

use crate::configuration::{Configuration, FileDownloadDefinition, FileDownloadOperation};
use crate::error::{Error, Result};

use super::run_command;

pub async fn execute_download_operations(config: &Configuration) -> Result<()> {
    match &config.file_downloads {
        Some(operations) => join_all(
            operations
                .iter()
                .map(|op| execute_download_operation(op, config)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<()>>>()
        .map(|_| ()),
        None => Ok(()),
    }
}

async fn execute_download_operation(
    operation: &FileDownloadOperation,
    config: &Configuration,
) -> Result<()> {
    let target = operation
        .download_target_base()
        .ok_or_else(|| Error::from("Unable to resolve target directory"))?;
    if !target.exists() {
        fs::create_dir_all(&target)?;
    }
    debug!("{:?}", target);
    let client: Client = Client::new();
    let results: Result<()> = join_all(
        operation
            .files
            .iter()
            .map(|fl| download_target(&fl, &target, &client)),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<()>>>()
    .map(|_| ());

    if results.is_ok() {
        match &operation.after_complete {
            Some(after) => run_command(after, config.system_details),
            None => results,
        }
    } else {
        results
    }
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
