use serde::Deserialize;
use std::fs::File;
use std::io::Read;

use crate::error::SpinupError;

// pub enum DistroPackages {
//     Arch(Vec<String>),
//     Debian(Vec<String>),
//     Ubuntu(Vec<String>),
// }

#[derive(Debug, Deserialize)]
pub struct DistroPackages {
    pub target_os: String,
    pub packages: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub packages: Option<Vec<String>>,
    pub distro_packages: Option<Vec<DistroPackages>>,
}

pub fn read_in_config(config_path: &str) -> Result<Configuration, SpinupError> {
    if let Ok(mut file) = File::open(config_path) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            let config = toml::from_str::<Configuration>(&contents).unwrap();
            return Ok(config);
        }
    }
    Err(SpinupError::ConfigurationReadError(String::from(
        config_path,
    )))
}
