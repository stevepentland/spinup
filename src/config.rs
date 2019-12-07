use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use dirs;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct DistroPackages {
    pub target_os: String,
    pub packages: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
    pub packages: Option<Vec<String>>,
    pub distro_packages: Option<Vec<DistroPackages>>,
    pub file_downloads: Option<Vec<FileDownloadOperation>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomCommand {
    pub command: String,
    pub args: Option<Vec<String>>,
}

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

pub fn read_in_config(config_path: &str) -> Result<Configuration> {
    PathBuf::from(config_path).canonicalize().map(|target| {
        if !target.is_file() {
            return Err(format!("{:?} is not a file", target).into());
        }

        let syntax_guess = guess_file_syntax(&target);
        File::open(target).map(|mut file| {
            let mut contents = String::new();
            file.read_to_string(&mut contents).map(|_| parse_file_contents(contents, syntax_guess))?
        })?
    })?
}

#[derive(Debug, PartialEq)]
enum FileSyntax {
    Toml,
    Yaml,
    Json,
    Unknown,
}

fn parse_file_contents(
    contents: String,
    assumed_syntax: FileSyntax,
) -> Result<Configuration> {
    match assumed_syntax {
        FileSyntax::Toml => toml::from_str(&contents).or_else(|e| Err(e.into())),
        FileSyntax::Yaml => serde_yaml::from_str(&contents).or_else(|e| Err(e.into())),
        FileSyntax::Json => serde_json::from_str(&contents).or_else(|e| Err(e.into())),
        _ => toml::from_str(&contents)
            .or_else(|_| serde_yaml::from_str(&contents))
            .or_else(|_| serde_json::from_str(&contents))
            .or_else(|_| Err(Error::Other(String::from("Was unable to parse config file contents using any syntax")))),
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
            .or_else(|| {
                env::current_dir().ok().or_else(dirs::home_dir)
            })
    }
}

#[cfg(test)]
mod tests {
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
}
