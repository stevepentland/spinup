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
    upgrade_system(&config)?;

    if let Some(packages) = &config.package_list {
        run_command(packages, config.system_details)
    } else {
        info!("No packages were detected in the configuration file");
        Ok(())
    }
}

fn upgrade_system(config: &Configuration) -> Result<()> {
    if !config.update_system {
        return Ok(());
    }

    let package_manager = config.system_details.package_manager();

    if let Some(ref update_cmd) = package_manager.update_operation() {
        run_command(update_cmd, config.system_details)?;
    }

    run_command(&package_manager.upgrade_operation(), config.system_details)
}
