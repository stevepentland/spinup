use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

mod command;
mod files;
mod packages;
mod snap;
mod system;

pub use command::*;
pub use files::*;
pub use packages::*;
pub use snap::*;
pub use system::*;

/// Main configuration struct
#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
    /// An optional list of [`PackageList`](struct.PackageList.html) items to install
    pub package_list: Option<PackageList>,

    /// An optional list of [`FileDownloadOperation`](struct.FileDownloadOperation) specifying files
    /// to download
    pub file_downloads: Option<Vec<FileDownloadOperation>>,

    /// An optional list of [`Snaps`](struct.Snaps.html) to install
    pub snaps: Option<Snaps>,

    /// The current system details when this configuration was created
    #[serde(skip, default = "SystemDetails::default")]
    pub system_details: SystemDetails,
}

/// Read in the configuration file specified by `config_path` and parse its contents
/// into a `Configuration` instance.
///
/// # Arguments:
/// - `config_path`: The path to the target configuration file
///
/// # Returns:
/// A [`Configuration`](struct.Configuration.html) instance
///
/// # Errors:
/// This function will return an error under the following conditions:
/// - The specified path does not exist
/// - The specified file cannot be read
/// - The specified file cannot be parsed as toml, yaml, or json
pub fn read_in_config(config_path: &str) -> Result<Configuration> {
    PathBuf::from(config_path).canonicalize().map(|target| {
        if !target.is_file() {
            return Err(format!("{:?} is not a file", target).into());
        }

        let syntax_guess = guess_file_syntax(&target);
        File::open(target).map(|mut file| {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map(|_| parse_file_contents(contents, syntax_guess))?
        })?
    })?
}

/// Possible syntax options a config file could be
#[derive(Debug, PartialEq)]
enum FileSyntax {
    Toml,
    Yaml,
    Json,
    Unknown,
}

fn parse_file_contents(contents: String, assumed_syntax: FileSyntax) -> Result<Configuration> {
    match assumed_syntax {
        FileSyntax::Toml => toml::from_str(&contents).or_else(|e| Err(e.into())),
        FileSyntax::Yaml => serde_yaml::from_str(&contents).or_else(|e| Err(e.into())),
        FileSyntax::Json => serde_json::from_str(&contents).or_else(|e| Err(e.into())),
        _ => toml::from_str(&contents)
            .or_else(|_| serde_yaml::from_str(&contents))
            .or_else(|_| serde_json::from_str(&contents))
            .or_else(|_| {
                Err(Error::Other(String::from(
                    "Was unable to parse config file contents using any syntax",
                )))
            }),
    }
}

fn guess_file_syntax(path: &Path) -> FileSyntax {
    if let Some(os_extension) = path.extension() {
        if let Some(extension) = os_extension.to_str() {
            return match &extension.to_lowercase()[..] {
                "toml" => FileSyntax::Toml,
                "yaml" | "yml" => FileSyntax::Yaml,
                "json" => FileSyntax::Json,
                _ => FileSyntax::Unknown,
            };
        }
    }
    FileSyntax::Unknown
}

impl FileDownloadOperation {
    pub fn download_target_base(&self) -> Option<PathBuf> {
        self.base_dir
            .as_ref()
            .map(PathBuf::from)
            .or_else(|| env::current_dir().ok().or_else(dirs::home_dir))
            .and_then(fixup_path)
    }
}

fn fixup_path(mut path: PathBuf) -> Option<PathBuf> {
    if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            path = path.strip_prefix("~").ok().map(|p| home.join(p))?;
        }
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_json_from_extension() {
        let p = Path::new("filename.json");
        let actual = guess_file_syntax(&p);
        assert_eq!(actual, FileSyntax::Json);
    }

    #[test]
    fn test_toml_from_extension() {
        let p = Path::new("filename.toml");
        let actual = guess_file_syntax(&p);
        assert_eq!(actual, FileSyntax::Toml);
    }

    #[test]
    fn test_yaml_from_full_extension() {
        let p = Path::new("filename.yaml");
        let actual = guess_file_syntax(&p);
        assert_eq!(actual, FileSyntax::Yaml);
    }

    #[test]
    fn test_yaml_from_short_extension() {
        let p = Path::new("filename.yml");
        let actual = guess_file_syntax(&p);
        assert_eq!(actual, FileSyntax::Yaml);
    }

    #[test]
    fn test_unknown_with_no_extension() {
        let p = Path::new("filename");
        let actual = guess_file_syntax(&p);
        assert_eq!(actual, FileSyntax::Unknown);
    }

    const TOML_DATA: &str = r#"
packages = [
    "package"
]
"#;

    const JSON_DATA: &str = r#"
{
    "packages": [
        "package",
        "package2"
    ],
    "distro_packages": [
        {
            "target_os": "arch",
            "packages": [
                "bat"
            ]
        }
    ]
}
"#;

    const YAML_DATA: &str = r#"
---
packages:
    - package

"#;

    #[test]
    fn test_toml_with_toml_guess_ok() {
        let actual = parse_file_contents(String::from(TOML_DATA), FileSyntax::Toml);
        assert!(actual.is_ok());
    }

    #[test]
    fn test_toml_with_yaml_guess_err() {
        let actual = parse_file_contents(String::from(TOML_DATA), FileSyntax::Yaml);
        assert!(actual.is_err());
    }

    #[test]
    fn test_toml_with_json_guess_err() {
        let actual = parse_file_contents(String::from(TOML_DATA), FileSyntax::Json);
        assert!(actual.is_err());
    }

    #[test]
    fn test_json_with_json_guess_ok() {
        let actual = parse_file_contents(String::from(JSON_DATA), FileSyntax::Json);
        assert!(actual.is_ok());
    }

    #[test]
    fn test_json_with_yaml_guess_err() {
        let actual = parse_file_contents(String::from(JSON_DATA), FileSyntax::Yaml);
        // Note: serde_yaml actually parses the json for some reason
        assert!(actual.is_ok());
    }

    #[test]
    fn test_json_with_toml_guess_err() {
        let actual = parse_file_contents(String::from(JSON_DATA), FileSyntax::Toml);
        assert!(actual.is_err());
    }

    #[test]
    fn test_yaml_with_yaml_guess_ok() {
        let actual = parse_file_contents(String::from(YAML_DATA), FileSyntax::Yaml);
        assert!(actual.is_ok());
    }

    #[test]
    fn test_yaml_with_json_guess_err() {
        let actual = parse_file_contents(String::from(YAML_DATA), FileSyntax::Json);
        assert!(actual.is_err());
    }

    #[test]
    fn test_yaml_with_toml_guess_err() {
        let actual = parse_file_contents(String::from(YAML_DATA), FileSyntax::Toml);
        assert!(actual.is_err());
    }

    #[test]
    fn test_valid_yaml_with_unknown_ok() {
        let actual = parse_file_contents(String::from(YAML_DATA), FileSyntax::Unknown);
        assert!(actual.is_ok());
    }

    #[test]
    fn test_valid_toml_with_unknown_ok() {
        let actual = parse_file_contents(String::from(TOML_DATA), FileSyntax::Unknown);
        assert!(actual.is_ok());
    }

    #[test]
    fn test_valid_json_with_unknown_ok() {
        let actual = parse_file_contents(String::from(JSON_DATA), FileSyntax::Unknown);
        assert!(actual.is_ok());
    }

    #[test]
    fn test_junk_data_unknown_still_error() {
        let actual = parse_file_contents(String::from("Somerandomjunk"), FileSyntax::Unknown);
        assert!(actual.is_err());
    }

    #[test]
    fn test_reading_in_good_toml_config() {
        let actual = read_in_config("./data/sample.toml");
        assert!(actual.is_ok());
    }

    #[test]
    fn test_reading_in_good_json_config() {
        let actual = read_in_config("./data/sample.json");
        assert!(actual.is_ok());
    }

    #[test]
    fn test_reading_in_good_yaml_config() {
        let actual = read_in_config("./data/sample.yml");
        assert!(actual.is_ok());
    }

    #[test]
    fn test_reading_fail_on_dir() {
        let actual = read_in_config("/tmp");
        assert!(actual.is_err());
    }

    #[test]
    fn test_guess_unknown_syntax() {
        let actual = guess_file_syntax(Path::new("./data/noextension"));
        assert_eq!(actual, FileSyntax::Unknown);
    }

    #[test]
    fn test_guess_unknown_from_unknown_extension() {
        let actual = guess_file_syntax(Path::new("./data/setup.sh"));
        assert_eq!(actual, FileSyntax::Unknown);
    }
}
