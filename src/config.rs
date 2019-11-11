use serde::Deserialize;
use std::fs::File;
use std::io::Read;

use crate::error::SpinupError;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub packages: Option<Vec<String>>,
}

pub fn read_in_config(config_path: &str) -> Result<Configuration, SpinupError> {
    if let Ok(mut file) = File::open(config_path) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            let config_r = toml::from_str::<Configuration>(&contents);
            if let Ok(config) = config_r {
                return Ok(config);
            }
        }
    }
    Err(SpinupError::ConfigurationReadError(String::from(
        config_path,
    )))
}
