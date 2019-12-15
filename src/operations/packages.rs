use super::run_command;

use crate::configuration::Configuration;
use crate::error::Result;

/// Call the system package manager to install the packages contained
/// in the configuration.
///
/// # Arguments
///
/// * `config` - The current configuration
/// * `details` - Details on the current system this process is running in
///
pub fn install_packages(config: &Configuration) -> Result<()> {
    if let Some(packages) = &config.package_list {
        run_command(packages, &config.system_details)
    } else {
        info!("No packages were detected in the configuration file");
        Ok(())
    }
}

/// TODO: See about using this in `run_command`
fn _handle_process_output(output: std::process::Output) -> Result<()> {
    if let Some(code) = output.status.code() {
        if code == 0 {
            info!("Package install process completed successfully");
            Ok(())
        } else {
            use log::{max_level, LevelFilter};
            warn!("Package install returned status of {}", code);

            // Don't bother building trace output unless we're actually using it
            if max_level() == LevelFilter::Trace {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.len() > 0 {
                    debug!("Stdout: \n{}", stdout);
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.len() > 0 {
                    debug!("Stderr: \n{}", stderr);
                }
            }
            Err(format!("Package manager returned status of '{}'.\nRun with higher verbosity to see more output", code).into())
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::configuration::{DistroPackages, SystemDetails};
}
