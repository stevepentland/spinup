//! The files module defines the structures responsible for
//! indicating files that should be downloaded when the app
//! runs.
//!
//! An operation consists of 3 different parts, which are
//! the target folder to download the files to, the files that
//! should be downloaded, and a command that should be run after
//! the download(s) finish.
use serde::{Deserialize, Serialize};

use super::CustomCommand;

/// A `FileDownloadDefinition` defines a file to download
/// and what to name it.
#[derive(Debug, Deserialize, Serialize)]
pub struct FileDownloadDefinition {
    /// The source of the file, this should be a publicly accessible URL.
    pub source: String,

    /// The target filename to download to, this name will be appended to [`base_dir`](struct.FileDownloadOperation.html#structfield.base_dir)
    pub target: String,
}

/// The container definition for a full set of file downloads.
#[derive(Debug, Deserialize, Serialize)]
pub struct FileDownloadOperation {
    /// The directory to download all files into
    pub base_dir: Option<String>,

    /// A command to run after all downloads complete
    pub after_complete: Option<CustomCommand>,

    /// The files to download
    pub files: Vec<FileDownloadDefinition>,
}
