use serde::{Deserialize, Serialize};

use super::CustomCommand;

#[derive(Debug, Deserialize, Serialize)]
pub struct FileDownloadDefinition {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileDownloadOperation {
    pub base_dir: Option<String>,
    pub after_complete: Option<CustomCommand>,
    pub files: Vec<FileDownloadDefinition>,
}
