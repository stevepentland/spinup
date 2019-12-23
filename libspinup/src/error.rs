use std::error;
use std::fmt;
use std::io;
use std::result;

use flexi_logger;
use reqwest;
use serde_json;
use serde_yaml;
use sys_info;
use toml::de;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum ConfigError {
    Json(serde_json::Error),
    Toml(de::Error),
    Yaml(serde_yaml::Error),
}

#[derive(Debug)]
pub enum Error {
    Config(ConfigError),
    SystemDetails(sys_info::Error),
    Io(io::Error),
    Request(reqwest::Error),
    Logger(flexi_logger::FlexiLoggerError),
    Other(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::Json(ref err) => err.fmt(f),
            ConfigError::Toml(ref err) => err.fmt(f),
            ConfigError::Yaml(ref err) => err.fmt(f),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Config(ref err) => err.fmt(f),
            Error::SystemDetails(ref err) => err.fmt(f),
            Error::Io(ref err) => err.fmt(f),
            Error::Request(ref err) => err.fmt(f),
            Error::Logger(ref err) => err.fmt(f),
            Error::Other(ref s) => write!(f, "{}", s),
        }
    }
}

impl error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ConfigError::Json(ref err) => Some(err),
            ConfigError::Toml(ref err) => Some(err),
            ConfigError::Yaml(ref err) => Some(err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Config(ref err) => err.source(),
            Error::SystemDetails(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            Error::Request(ref err) => Some(err),
            Error::Logger(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Request(err)
    }
}

impl From<sys_info::Error> for Error {
    fn from(err: sys_info::Error) -> Error {
        Error::SystemDetails(err)
    }
}

impl From<de::Error> for ConfigError {
    fn from(err: de::Error) -> ConfigError {
        ConfigError::Toml(err)
    }
}

impl From<de::Error> for Error {
    fn from(err: de::Error) -> Error {
        Error::Config(err.into())
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> ConfigError {
        ConfigError::Json(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Config(err.into())
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> ConfigError {
        ConfigError::Yaml(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Error {
        Error::Config(err.into())
    }
}

impl From<flexi_logger::FlexiLoggerError> for Error {
    fn from(err: flexi_logger::FlexiLoggerError) -> Error {
        Error::Logger(err)
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Error {
        Error::Other(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Error {
        Error::Other(String::from(msg))
    }
}
