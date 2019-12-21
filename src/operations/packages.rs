use super::run_command;

use crate::configuration::Configuration;
use crate::error::Result;

/// Call the system package manager to install the packages contained
/// in the configuration.
///
/// # Arguments
///
/// * `config` - The current configuration
pub fn install_packages(config: &Configuration) -> Result<()> {
    if let Some(packages) = &config.package_list {
        run_command(packages, config.system_details)
    } else {
        info!("No packages were detected in the configuration file");
        Ok(())
    }
}
