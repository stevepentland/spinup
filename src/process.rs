use libc;
use std::process::{Command, Stdio};

use crate::config::Configuration;
use crate::error::SpinupError;
use crate::system::SystemDetails;

/// Call the system package manager to install the packages contained
/// in the configuration.
///
/// # Arguments
///
/// * `config` - The current configuration
/// * `details` - Details on the current system this process is running in
///
/// # Example
///
/// ```rust
/// let config = Configuration { packages: Some(vec!["ripgrep", "exa"]) };
/// let details = SystemDetails { target_os: TargetOperatingSystem::Arch };
///
/// install_packages(&config, &details);
/// ```
///
pub fn install_packages(
    config: &Configuration,
    details: &SystemDetails,
) -> Result<(), SpinupError> {
    if let Some(packages) = &config.packages {
        log_package_info(packages);
        if let Some(pm) = details.package_manager() {
            trace!("Using package manager: {:?}", pm);
            // TODO: Look into directly using the local libraries (libalpm on Arch, etc)
            let mut command = Command::new(&pm.name);
            if let Some(install_command) = &pm.install_subcommand {
                command.arg(install_command);
            }
            if let Some(autoconfirm) = &pm.autoconfirm {
                command.arg(autoconfirm);
            }
            info!("Starting install of {} packages", packages.len());
            let res = command
                .args(packages)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match res {
                Ok(result) => match result.wait_with_output() {
                    Ok(output) => handle_process_output(output),
                    Err(o) => {
                        error!("Error while running install command: {:?}", o);
                        Err(SpinupError::ChildProcessSpawnError)
                    }
                },
                Err(e) => {
                    error!("Error encountered while spawning child process: {:?}", e);
                    Err(SpinupError::ChildProcessSpawnError)
                }
            }
        } else {
            error!(
                "Unable to find package manager for {:?}",
                details.current_os()
            );
            Err(SpinupError::NoPackageManagerForPlatform)
        }
    } else {
        warn!("No packages were detected in the configuration file");
        Ok(())
    }
}

fn handle_process_output(output: std::process::Output) -> Result<(), SpinupError> {
    if let Some(code) = output.status.code() {
        if code == 0 {
            info!("Package install process completed successfully");
            Ok(())
        } else {
            use log::{max_level, LevelFilter};
            info!("Package install returned status of {}", code);

            // Don't bother building trace output unless we're actually using it
            if max_level() == LevelFilter::Trace {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.len() > 0 {
                    trace!("Stdout: \n{}", stdout);
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.len() > 0 {
                    // Maybe stderr should always be shown?
                    trace!("Stderr: \n{}", stderr);
                }
            }
            Err(SpinupError::PackageInstallError(code))
        }
    } else {
        Ok(())
    }
}

fn log_package_info(packages: &[String]) {
    debug!("Have a total of {} packages to install", packages.len());
    trace!("Packages: {:?}", packages);
}

pub fn process_is_root() -> bool {
    unsafe {
        let uid = libc::getuid();
        uid == 0
    }
}
