use libc;
use std::process::Command;

use crate::{config::Configuration, error::SpinupError, system::SystemDetails};

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
        if let Some(pm) = details.package_manager() {
            // TODO: Look into directly using the local libraries (libalpm on Arch, etc)
            let mut c = Command::new(&pm.name);
            if let Some(install_command) = &pm.install_subcommand {
                c.arg(install_command);
            }
            if let Some(autoconfirm) = &pm.autoconfirm {
                c.arg(autoconfirm);
            }
            c.args(packages);
            let res = c.spawn();
            match res {
                Ok(r) => match r.wait_with_output() {
                    Ok(out) => {
                        println!("{:?}", out.status.code());
                    }
                    Err(_o) => {}
                },
                Err(_e) => println!("Err"),
            }
        }
        Ok(())
    } else {
        // Print out log message when logging setup
        Ok(())
    }
}

pub fn process_is_root() -> bool {
    unsafe {
        let uid = libc::getuid();
        uid == 0
    }
}
