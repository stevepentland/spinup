use libc;
use std::process::{Command, Stdio};

use crate::config::Configuration;
use crate::error::SpinupError;
use crate::system::{PackageManager, SystemDetails, TargetOperatingSystem};

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
/// ```ignore
/// # use crate::config::Configuration;
/// # use crate::system::SystemDetails;
/// let config = Configuration {
///     packages: Some(vec!["ripgrep", "exa"]),
///     distro_packages: Vec::new(),
/// };
/// let details = SystemDetails { target_os: TargetOperatingSystem::Arch };
///
/// install_packages(&config, &details);
/// ```
///
pub fn install_packages(
    config: &Configuration,
    details: &SystemDetails,
) -> Result<(), SpinupError> {
    if let Some(packages) = extract_packages(config, details) {
        log_package_info(&packages);
        if let Some(pm) = details.package_manager() {
            trace!("Using package manager: {:?}", pm);
            // TODO: Look into directly using the local libraries (libalpm on Arch, etc)
            let mut command = build_command(&packages, &pm);
            info!("Starting install of {} packages", packages.len());

            let res = command.spawn();
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

fn extract_packages(config: &Configuration, details: &SystemDetails) -> Option<Vec<String>> {
    let mut all_packages = Vec::new();
    if let Some(base_packages) = &config.packages {
        all_packages.extend_from_slice(base_packages);
    }
    if let Some(distro_packages) = &config.distro_packages {
        if let Some(this_distro) = distro_packages
            .iter()
            .find(|&elem| TargetOperatingSystem::from(&elem.target_os[..]) == details.current_os())
        {
            if let Some(pkgs) = &this_distro.packages {
                all_packages.extend_from_slice(pkgs);
            }
        }
    }

    if all_packages.is_empty() {
        None
    } else {
        Some(all_packages)
    }
}

fn build_command(packages: &[String], package_manager: &PackageManager) -> Command {
    let mut command = Command::new(&package_manager.name);
    if let Some(install_command) = &package_manager.install_subcommand {
        command.arg(install_command);
    }
    if let Some(autoconfirm) = &package_manager.autoconfirm {
        command.arg(autoconfirm);
    }
    command
        .args(packages)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());
    command
}

pub fn process_is_root() -> bool {
    unsafe {
        let uid = libc::getuid();
        uid == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DistroPackages;

    #[test]
    fn test_extract_empty_packages_is_none() {
        let config = Configuration {
            packages: None,
            distro_packages: None,
        };

        let details = SystemDetails::new(TargetOperatingSystem::Arch);

        let actual = extract_packages(&config, &details);
        assert_eq!(actual, None);
    }

    #[test]
    fn test_extract_only_base_packages_works() {
        let packages: Vec<String> = vec!["first".into(), "second".into()];
        let config = Configuration {
            packages: Some(packages.clone()),
            distro_packages: None,
        };
        let details = SystemDetails::new(TargetOperatingSystem::Arch);
        let actual = extract_packages(&config, &details).unwrap();
        assert_eq!(actual, packages);
    }

    #[test]
    fn test_extract_base_and_distro_works() {
        let packages: Vec<String> = vec!["first".into(), "second".into()];
        let distro_pkgs: Vec<String> = vec!["third".into(), "fourth".into()];
        let config = Configuration {
            packages: Some(packages.clone()),
            distro_packages: Some(vec![DistroPackages {
                target_os: "manjaro".into(),
                packages: Some(distro_pkgs.clone()),
            }]),
        };
        let details = SystemDetails::new(TargetOperatingSystem::Arch);
        let actual = extract_packages(&config, &details).unwrap();
        let expected = packages
            .into_iter()
            .chain(distro_pkgs.into_iter())
            .collect::<Vec<String>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_extract_ignores_not_current_os_single() {
        let packages: Vec<String> = vec!["first".into(), "second".into()];
        let distro_pkgs: Vec<String> = vec!["third".into(), "fourth".into()];
        let config = Configuration {
            packages: Some(packages.clone()),
            distro_packages: Some(vec![DistroPackages {
                target_os: "ubuntu".into(),
                packages: Some(distro_pkgs.clone()),
            }]),
        };
        let details = SystemDetails::new(TargetOperatingSystem::Arch);
        let actual = extract_packages(&config, &details).unwrap();
        assert_eq!(actual, packages);
    }

    #[test]
    fn test_extract_only_takes_current_os_target() {
        let packages: Vec<String> = vec!["first".into(), "second".into()];
        let distro_pkgs: Vec<String> = vec!["third".into(), "fourth".into()];
        let arch_pkgs: Vec<String> = vec!["fifth".into(), "sixth".into()];
        let config = Configuration {
            packages: Some(packages.clone()),
            distro_packages: Some(vec![
                DistroPackages {
                    target_os: "ubuntu".into(),
                    packages: Some(distro_pkgs.clone()),
                },
                DistroPackages {
                    target_os: "linuxmint".into(),
                    packages: Some(distro_pkgs.clone()),
                },
                DistroPackages {
                    target_os: "arch".into(),
                    packages: Some(arch_pkgs.clone()),
                },
            ]),
        };
        let details = SystemDetails::new(TargetOperatingSystem::Arch);
        let actual = extract_packages(&config, &details).unwrap();
        let expected = packages
            .into_iter()
            .chain(arch_pkgs.into_iter())
            .collect::<Vec<String>>();
        assert_eq!(actual, expected);
    }
}
