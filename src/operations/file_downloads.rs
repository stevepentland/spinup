use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use futures::future::join_all;
use reqwest::Client;

use crate::config::{Configuration, FileDownloadDefinition, FileDownloadOperation};
use crate::error::{Error, Result};

pub async fn execute_download_operations(config: &Configuration) -> Result<()> {
    match &config.file_downloads {
        Some(operations) => join_all(operations.iter().map(|op| execute_download_operation(op)))
            .await
            .into_iter()
            .collect::<Result<Vec<()>>>()
            .map(|_| ()),
        None => Ok(()),
    }
}

async fn execute_download_operation(operation: &FileDownloadOperation) -> Result<()> {
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
        // TODO: Since commands will be run from multiple modules, it may be prudent to centralize command running
        match &operation.after_complete {
            Some(after) => {
                let mut base = after.command.clone();
                if let Some(args) = &after.args {
                    for arg in args.iter() {
                        base.push_str(&format!(" {}", arg));
                    }
                }
                match Command::new("sh").arg("-c").arg(&base).spawn()?.wait() {
                    Ok(status) => {
                        if status.success() {
                            Ok(())
                        } else {
                            let status_code = {
                                match status.code() {
                                    Some(code) => code.to_string(),
                                    None => "unknown".to_string(),
                                }
                            };

                            Err(Error::from(
                                    format!(
                                        "Received  {} status code when running after-download command: '{}'", 
                                        &status_code,
                                        &base
                                    )
                                )
                            )
                        }
                    }
                    Err(e) => Err(Error::from(e)),
                }
            }
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
